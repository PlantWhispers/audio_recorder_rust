use crate::shared_buffer::{SharedBuffer, SharedBufferMessage::{NewFile, Data, EndOfFile, EndThread}};
use std::sync::Arc;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom, Result};

// Import the constants
use crate::SAMPLE_RATE;
use crate::CHANNELS;

fn write_wav_header(file: &mut File, num_channels: u16, sample_rate: u32, bits_per_sample: u16) -> std::io::Result<()> {
    let byte_rate = sample_rate as u32 * num_channels as u32 * bits_per_sample as u32 / 8;
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
        println!("Closed file");
    }
    Ok(())
}

pub fn write_audio(shared_buffer: Arc<SharedBuffer>) -> Result<()> {
    println!("Writing thread started");

    let mut file: Option<File> = None;

    loop {
        match shared_buffer.pull() {
            None => continue,
            Some(EndThread) => {
                end_file(&mut file)?;
                println!("Writing thread stopped");
                break;
            }
            Some(NewFile(filename)) => {
                end_file(&mut file)?; // Close the previous file (if any)
                file = Some(File::create(filename)?);
                write_wav_header(&mut file.as_mut().unwrap(), CHANNELS, SAMPLE_RATE, 16)?; //TODO: Bits per sample is hardcoded
            }
            Some(Data(data)) => {
                for sample in data {
                    file.as_mut().unwrap().write_all(&sample.to_le_bytes())?;
                }
            }
            Some(EndOfFile) => end_file(&mut file)?,
        }
    }

    Ok(())
}
