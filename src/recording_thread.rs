use crate::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
use crate::pcm_setup::setup_pcm;
use crate::{BUFFER_SIZE, N_OF_BUFFERS_PER_FILE, SAMPLE_RATE};
use alsa::pcm::{IO, PCM};
use alsa::PollDescriptors;
use crossbeam::channel::Sender;
use libc::{poll, pollfd};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

pub fn recording_thread_logic(
    sender: Sender<RecorderToWriterChannelMessage>,
    shutdown_signal: Arc<AtomicBool>,
) {
    let pcm_devices = setup_pcm().unwrap();

    pcm_devices[0].link(&pcm_devices[1]).unwrap();

    pcm_devices[0].start().unwrap();

    let pcm_ios = pcm_devices
        .iter()
        .map(|device| device.io_i16().unwrap())
        .collect::<Vec<_>>();

    let mut pds: Vec<pollfd> = pcm_devices
        .iter()
        .map(|device| {
            let mut fd = PollDescriptors::get(device).unwrap()[0];
            fd.events = libc::POLLIN;
            fd.revents = 0;
            fd
        })
        .collect();

    enum First {
        Empty,
        A(Instant),
        B(Instant),
    }
    use First::*;

    'outer: while !shutdown_signal.load(Ordering::SeqCst) {
        sender.send(NewFile(new_file_name())).unwrap();

        let mut first: First = Empty;

        pcm_devices[0].reset().unwrap();
        pcm_devices[1].reset().unwrap();

        let end_time = loop {
            unsafe { poll(pds.as_mut_ptr(), pds.len() as libc::nfds_t, 1000) };
            let time = std::time::Instant::now();
            let a_has_data = pds[0].revents > 0;
            let b_has_data = pds[1].revents > 0;

            if a_has_data == b_has_data {
                if a_has_data {
                    break time; // Both devices have data
                }
                continue; // No data on either device
            }

            if let Empty = first {
                first = if a_has_data { A(time) } else { B(time) };
            }
        };

        match first {
            Empty => {}
            A(start) => {
                let delay = end_time.duration_since(start);
                let delay_in_frames: i64 =
                    (delay.as_secs_f64() * SAMPLE_RATE as f64).round() as i64;
                let mut temp_buffer = vec![0i16; delay_in_frames as usize];
                pcm_ios[0].readi(&mut temp_buffer).unwrap();
                println!("B was {:?} (~{} frames) later", delay, delay_in_frames);
            }
            B(start) => {
                let delay = end_time.duration_since(start);
                let delay_in_frames: i64 =
                    (delay.as_secs_f64() * SAMPLE_RATE as f64).round() as i64;
                let mut temp_buffer = vec![0i16; delay_in_frames as usize];
                pcm_ios[1].readi(&mut temp_buffer).unwrap();
                println!("A was {:?} (~{} frames) later", delay, delay_in_frames);
            }
        }

        for _ in 0..N_OF_BUFFERS_PER_FILE {
            let data = {
                (
                    get_mic_data(&pcm_devices[0], &pcm_ios[0]),
                    get_mic_data(&pcm_devices[1], &pcm_ios[1]),
                )
            };
            match data {
                (Ok(a), Ok(b)) => {
                    sender.send(Data([a, b])).unwrap();
                }
                _ => continue 'outer,
            }
        }
    }
    sender.send(EndThread).unwrap();
}

fn get_mic_data(pcm_device: &PCM, pcm_io: &IO<'_, i16>) -> Result<Vec<i16>, Box<dyn Error>> {
    let mut buffer = vec![0i16; BUFFER_SIZE];
    match pcm_io.readi(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => {
            if pcm_device.try_recover(err, false).is_err() {
                panic!("Failed to recover from ALSA error: {}", err);
            }
            Err(err.into())
        }
    }
}

fn new_file_name() -> String {
    format!(
        "recordings/{}.wav",
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
}
