use crate::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
use crate::{BUFFER_SIZE, N_OF_BUFFERS_PER_FILE, SAMPLE_RATE};
use alsa::pcm::{IO, PCM};
use alsa::PollDescriptors;
use crossbeam::channel::Sender;
use libc::pollfd;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

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

        for pcm_device in pcm_devices.iter() {
            match pcm_device.reset() {
                Ok(_) => {}
                Err(err) => {
                    pcm_device.try_recover(err, false).unwrap();
                    continue 'outer;
                }
            }
        }

        while (unsafe { libc::poll(pds.as_mut_ptr(), 2, 1000) } != 1) {}

        let time = std::time::Instant::now();

        while (unsafe { libc::poll(pds.as_mut_ptr(), 2, 1000) } != 2) {}

        let delay = time.elapsed();
        let frame_delay: i64 = (delay.as_secs_f64() * SAMPLE_RATE as f64).round() as i64;

        println!("Delay: {:?} (~{} frames)", delay, frame_delay);

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
