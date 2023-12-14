pub enum SharedBufferMessage {
    NewFile(String),
    Data([[i16; 1920]; 2]),
    EndThread,
}
