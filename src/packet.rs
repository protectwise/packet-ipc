use ipc_channel::{
    ipc::IpcSharedMemory
};
use serde_derive::{Serialize, Deserialize};

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Packet {
    ts: std::time::SystemTime,
    data: IpcSharedMemory
}

impl Packet {
    pub fn timestamp(&self) -> &std::time::SystemTime { &self.ts }
    pub fn data(&self) -> &[u8] { &self.data }

    pub fn new<T: AsRef<[u8]>>(ts: std::time::SystemTime, data: T) -> Packet {
        Packet {
            ts: ts,
            data: IpcSharedMemory::from_bytes(data.as_ref())
        }
    }

    pub fn from_raw_parts(ts: std::time::SystemTime, len: usize, ptr: *const u8) -> Packet {
        if ptr == std::ptr::null() {
            panic!("Passed a null pointer to Packet::new");
        }
        let s = unsafe {
            std::slice::from_raw_parts(ptr, len)
        };
        Packet {
            ts: ts,
            data: IpcSharedMemory::from_bytes(s)
        }
    }

    pub fn into_raw(self) -> *mut Packet {
        Box::new(self).into_raw()
    }

    pub fn from_raw(p: *mut Packet) -> Box<Packet> {
        unsafe {
            Box::from_raw(p)
        }
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        println!("Drop occurred");
    }
}

unsafe impl Send for Packet {}

pub trait AsIpcPacket {
    fn timestamp(&self) -> &std::time::SystemTime;
    fn data(&self) -> &[u8];

    fn as_ipc_packet(&self) -> Packet {
        Packet::new(self.timestamp().clone(), self.data())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_into_raw() {
        let packet = Packet::new(std::time::UNIX_EPOCH, vec![0; 4]);
    }
}