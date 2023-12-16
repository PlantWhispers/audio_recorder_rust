use crate::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
use crate::SAMPLE_RATE;
use crossbeam::channel::Receiver;
use std::fs::File;
use std::io::{BufWriter, Result, Seek, SeekFrom, Write};

const BITS_PER_SAMPLE: u16 = 16;
const NUM_CHANNELS_IN_FILE: u16 = 2;

fn write_wav_header(
    file: &mut File,
    num_channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
) -> std::io::Result<()> {
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let sub_chunk2_size: u32 = 0; // Placeholder since we don't know the size yet
    let chunk_size: u32 = 36 + sub_chunk2_size; // --//--

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

fn end_file(file: &mut Option<BufWriter<File>>) -> std::io::Result<()> {
    if let Some(mut buf_file) = file.take() {
        buf_file.flush()?; // Ensure all data is written to disk
        let mut inner_file = buf_file.into_inner()?; // Get the underlying File
        update_wav_header(&mut inner_file)?; // Update the header with the correct file size
    }
    Ok(())
}

pub fn writing_thread_logic(receiver: Receiver<RecorderToWriterChannelMessage>) -> Result<()> {
    let mut file: Option<BufWriter<File>> = None;

    for message in receiver {
        match message {
            EndThread => {
                end_file(&mut file)?; // TODO: Error handling
                break;
            }
            NewFile(filename) => {
                end_file(&mut file)?; // Close the previous file (if any) // TODO: Error handling
                println!("Creating new file: {}", &filename);
                let mut new_file = File::create(filename)?; // TODO: Error handling (retry if file can't be created)
                write_wav_header(
                    &mut new_file,
                    NUM_CHANNELS_IN_FILE,
                    SAMPLE_RATE,
                    BITS_PER_SAMPLE,
                )?; // TODO: Error handling
                file = Some(BufWriter::new(new_file));
            }
            Data(data) => {
                if let Some(ref mut writer) = file {
                    let mut buffer = Vec::new();
                    for (a, b) in data[0].iter().zip(data[1].iter()) {
                        buffer.extend_from_slice(&a.to_le_bytes());
                        buffer.extend_from_slice(&b.to_le_bytes());
                    }
                    writer.write_all(&buffer)?; // TODO: Error handling
                }
            }
        }
    }

    Ok(())
}
