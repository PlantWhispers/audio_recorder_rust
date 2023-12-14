use crossbeam::channel::Receiver;

use crate::shared_buffer::SharedBufferMessage::{self, Data, EndThread, NewFile};
use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};

// Import the constants
// use crate::CHANNELS;
use crate::SAMPLE_RATE;

fn write_wav_header(
    file: &mut File,
    num_channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
) -> std::io::Result<()> {
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let sub_chunk2_size: u32 = 0; // Placeholder for now, specify the type explicitly
    let chunk_size: u32 = 36 + sub_chunk2_size; // specify the type explicitly

    file.write_all(b"RIFF")?;
    file.write_all(&chunk_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // Sub-chunk1Size
    file.write_all(&1u16.to_le_bytes())?; // AudioFormat
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&sub_chunk2_size.to_le_bytes())?;

    Ok(())
}

fn update_wav_header(file: &mut File) -> std::io::Result<()> {
    let file_size = file.seek(SeekFrom::End(0))? as u32;
    let chunk_size = file_size - 8;
    let sub_chunk2_size = file_size - 44;

    file.seek(SeekFrom::Start(4))?;
    file.write_all(&chunk_size.to_le_bytes())?;
    file.seek(SeekFrom::Start(40))?;
    file.write_all(&sub_chunk2_size.to_le_bytes())?;

    Ok(())
}

fn end_file(file: &mut Option<File>) -> std::io::Result<()> {
    if let Some(mut file) = file.take() {
        update_wav_header(&mut file)?;
    }
    Ok(())
}

pub fn write_audio(receiver: Receiver<SharedBufferMessage>) -> Result<()> {
    let mut file: Option<File> = None;

    for message in receiver {
        match message {
            EndThread => {
                end_file(&mut file)?;
                break;
            }
            NewFile(filename) => {
                end_file(&mut file)?; // Close the previous file (if any)
                file = Some(File::create(filename)?);
                write_wav_header(file.as_mut().unwrap(), 2, SAMPLE_RATE, 16)?;
                //TODO: Bits per sample is hardcoded
            }
            Data(data) => {
                // Write the data interleaved to the file
                for (a, b) in data[0].iter().zip(data[1].iter()) {
                    file.as_mut().unwrap().write_all(&a.to_le_bytes())?;
                    file.as_mut().unwrap().write_all(&b.to_le_bytes())?;
                }
            }
        }
    }

    Ok(())
}
