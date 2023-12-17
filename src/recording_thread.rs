use crate::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
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
    pcm_devices: [PCM; 2],
    sender: Sender<RecorderToWriterChannelMessage>,
    shutdown_signal: Arc<AtomicBool>,
) {
    pcm_devices[0].start().unwrap();
    pcm_devices[1].start().unwrap();

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

    'outer: while !shutdown_signal.load(Ordering::SeqCst) {
        sender.send(NewFile(new_file_name())).unwrap();

        let mut start_time: Option<Instant> = None;
        let mut end_time: Option<Instant> = None;
        enum AorB {
            A,
            B,
        }
        let mut first: Option<AorB> = None;

        for pcm_device in pcm_devices.iter() {
            match pcm_device.reset() {
                Ok(_) => {}
                Err(err) => {
                    pcm_device.try_recover(err, false).unwrap();
                    continue 'outer;
                }
            }
        }

        while end_time.is_none() {
            unsafe { poll(pds.as_mut_ptr(), pds.len() as libc::nfds_t, 1000) };
            let time = std::time::Instant::now();

            match (pds[0].revents > 0, pds[1].revents > 0, &first.is_none()) {
                (false, false, true) | (true, false, false) | (false, true, false) => continue,
                (true, false, true) => {
                    first = Some(AorB::A);
                    start_time = Some(time);
                }
                (false, true, true) => {
                    first = Some(AorB::B);
                    start_time = Some(time);
                }
                (true, true, false) => {
                    end_time = Some(time);
                }
                (true, true, true) => {
                    panic!("Both mics started at the same time")
                }
                (false, false, false) => {
                    panic!("Invalid state")
                }
            }
        }

        let delay = match (start_time, end_time) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => panic!("Timing failed"),
        };

        let frame_delay: i64 = (delay.as_secs_f64() * SAMPLE_RATE as f64).round() as i64;

        println!(
            "{:?} was {:?} (~{} frames) faster",
            match first.as_ref().unwrap() {
                AorB::A => "Mic A",
                AorB::B => "Mic B",
            },
            delay,
            frame_delay
        );

        let mut temp_buffer = vec![0i16; frame_delay as usize];

        match first.unwrap() {
            AorB::A => {
                pcm_ios[0].readi(&mut temp_buffer).unwrap();
            }
            AorB::B => {
                pcm_ios[1].readi(&mut temp_buffer).unwrap();
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
