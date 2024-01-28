pub enum RecorderToWriterChannelMessage {
    NewFile(String),
    Data([Vec<i16>; 2]),
    EndThread,
}
