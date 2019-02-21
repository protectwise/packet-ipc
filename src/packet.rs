use serde_derive::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Packet {
    ts: std::time::SystemTime,
    data: Vec<u8>
}

impl Packet {
    pub fn timestamp(&self) -> &std::time::SystemTime {
        &self.ts
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn new(ts: std::time::SystemTime, data: Vec<u8>) -> Packet {
        Packet {
            ts,
            data
        }
    }
}
