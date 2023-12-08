use alsa::{pcm::{PCM, HwParams, Format, Access}, Direction, direct::pcm};
use std::io::{Write, Seek, SeekFrom, Read};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the default recording device
    let pcm_a = PCM::new("hw:1,0", Direction::Capture, false)?;
    let pcm_b = PCM::new("hw:2,0", Direction::Capture, false)?;

    // Set hardware parameters: 384 kHz / Mono / 16 bit
    let hwp = HwParams::any(&pcm_a)?;
    hwp.set_channels(1)?;
    hwp.set_rate(384_000, alsa::ValueOr::Nearest)?;
    hwp.set_buffer_size(768000)?;
    hwp.set_format(Format::S16LE)?;
    hwp.set_access(Access::RWInterleaved)?;
    pcm_a.hw_params(&hwp)?;
    pcm_b.hw_params(&hwp)?;

    // TODO: make sure dir exists
    // Use current timestamp to create a unique file name
    let filename_a = format!("{}[a].wav", chrono::Utc::now().timestamp());
    let filename_b = format!("{}[b].wav", chrono::Utc::now().timestamp());
    // Create and open a file for writing
    let mut file_a = File::create("recordings/".to_owned() + &filename_a)?;
    let mut file_b = File::create("recordings/".to_owned() + &filename_b)?;

    
    // Write WAV header placeholder
    write_wav_header(&mut file_a, 0, 384_000, 16, 1)?;
    write_wav_header(&mut file_b, 0, 384_000, 16, 1)?;

    let mut frame_buf_a = [0i16; 1920];
    let mut frame_buf_b = [0i16; 1920];
    let mut buf_a = Vec::new();
    let mut buf_b = Vec::new();

    // Record audio
    pcm_a.prepare()?;
    pcm_b.prepare()?;
    pcm_a.start()?;
    pcm_b.start()?;
    for _ in 0..(5 * 384_000 / 1920) {
        
        pcm_a.io_i16()?.readi(&mut frame_buf_a)?;
        pcm_b.io_i16()?.readi(&mut frame_buf_b)?;

        println!("a: {:?}", pcm_a.avail()?);
        println!("b: {:?}", pcm_b.avail()?);

        // Append to buffer
        buf_a.extend_from_slice(&frame_buf_a);
        buf_b.extend_from_slice(&frame_buf_b);
    }
    print!("a: {:?}", pcm_a.state());
    pcm_a.drain()?;
    pcm_b.drain()?;

    // Write buffer to file
    for frame in buf_a.iter() {
        file_a.write_all(&frame.to_le_bytes())?;
    }
    for frame in buf_b.iter() {
        file_b.write_all(&frame.to_le_bytes())?;
    }


    // Update WAV header with the actual file size
    update_wav_header(&mut file_a)?;
    update_wav_header(&mut file_b)?;

    Ok(())
}

fn write_wav_header(file: &mut File, num_samples: u32, sample_rate: u32, bits_per_sample: u16, num_channels: u16) -> std::io::Result<()> {
    let byte_rate = sample_rate * u32::from(bits_per_sample) * u32::from(num_channels) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let sub_chunk2_size = num_samples * u32::from(block_align);
    let chunk_size = 36 + sub_chunk2_size;

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
