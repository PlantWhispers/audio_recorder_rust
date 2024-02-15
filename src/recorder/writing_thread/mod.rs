mod wav_utils;
use super::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
use crate::config::{SAMPLE_RATE, SOUNDFOLDER_PATH};
use crossbeam::channel::Receiver;
use std::fs::File;
use std::io::{BufWriter, Result, Write};
use wav_utils::{end_file, write_wav_header};

const BITS_PER_SAMPLE: u16 = 16;
const NUM_CHANNELS_IN_FILE: u16 = 2;
const TEMP_FILE_NAME: &str = ".temp.raw";

pub fn writing_thread_logic(receiver: Receiver<RecorderToWriterChannelMessage>) -> Result<()> {
    let mut file: Option<(BufWriter<File>, String)> = None;
    let temp_file_path = format!("{}{}", SOUNDFOLDER_PATH, TEMP_FILE_NAME);

    for message in receiver {
        match message {
            EndThread => {
                end_file(&mut file, &temp_file_path)?;
                break;
            }
            NewFile(filename) => {
                end_file(&mut file, &temp_file_path)?; // Close the previous file (if any)
                let mut new_file = File::create(&temp_file_path)?;
                write_wav_header(
                    &mut new_file,
                    NUM_CHANNELS_IN_FILE,
                    SAMPLE_RATE,
                    BITS_PER_SAMPLE,
                )?;
                file = Some((BufWriter::new(new_file), filename.clone()));
            }
            Data(data) => {
                if let Some((ref mut writer, _)) = file {
                    let mut buffer = Vec::new();
                    for (a, b) in data[0].iter().zip(data[1].iter()) {
                        buffer.extend_from_slice(&a.to_le_bytes());
                        buffer.extend_from_slice(&b.to_le_bytes());
                    }
                    writer.write_all(&buffer)?;
                }
            }
        }
    }

    Ok(())
}
