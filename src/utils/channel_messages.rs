use std::path::PathBuf;

pub enum RecorderToWriterChannelMessage {
    NewFile(PathBuf),
    Data([Vec<i16>; 2]),
    EndThread,
}
