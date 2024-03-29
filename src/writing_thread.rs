use crate::config::SAMPLE_RATE;
use crate::utils::channel_messages::RecorderToWriterChannelMessage::{
    self, Data, EndThread, NewFile,
};
use crate::utils::wav_utils::{end_file, write_wav_header};
use crossbeam::channel::Receiver;
use std::fs::File;
use std::io::{BufWriter, Result, Write};

const BITS_PER_SAMPLE: u16 = 16;
const NUM_CHANNELS_IN_FILE: u16 = 2;

pub fn writing_thread_logic(receiver: Receiver<RecorderToWriterChannelMessage>) -> Result<()> {
    let mut file: Option<BufWriter<File>> = None;
    // let temp_file_path = format!("{}{}", SOUNDFOLDER_PATH, TEMP_FILE_NAME);

    for message in receiver {
        match message {
            EndThread => {
                end_file(&mut file)?;
                break;
            }
            NewFile(path) => {
                end_file(&mut file)?; // Close the previous file (if any)
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut new_file = File::create(path)?;
                write_wav_header(
                    &mut new_file,
                    NUM_CHANNELS_IN_FILE,
                    SAMPLE_RATE,
                    BITS_PER_SAMPLE,
                )?;
                file = Some(BufWriter::new(new_file));
            }
            Data(data) => {
                let Some(ref mut writer) = file else {
                    continue;
                };

                let mut buffer = Vec::new();
                for (a, b) in data[0].iter().zip(data[1].iter()) {
                    buffer.extend_from_slice(&a.to_le_bytes());
                    buffer.extend_from_slice(&b.to_le_bytes());
                }
                writer.write_all(&buffer)?;
            }
        }
    }

    Ok(())
}
