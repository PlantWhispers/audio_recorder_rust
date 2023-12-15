pub enum SharedBufferMessage {
    NewFile(String),
    Data([Vec<i16>; 2]),
    EndThread,
}
