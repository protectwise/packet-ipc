use ipc_channel::ipc::IpcSharedMemory;
use serde_derive::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Packet {
    ts: std::time::SystemTime,
    data: IpcSharedMemory,
}

impl Packet {
    pub fn timestamp(&self) -> &std::time::SystemTime {
        &self.ts
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn new<T: AsRef<[u8]>>(ts: std::time::SystemTime, data: T) -> Packet {
        Packet {
            ts: ts,
            data: IpcSharedMemory::from_bytes(data.as_ref()),
        }
    }

    pub fn from_raw_parts(ts: std::time::SystemTime, len: usize, ptr: *const u8) -> Packet {
        if ptr == std::ptr::null() {
            panic!("Passed a null pointer to Packet::new");
        }
        let s = unsafe { std::slice::from_raw_parts(ptr, len) };
        Packet {
            ts: ts,
            data: IpcSharedMemory::from_bytes(s),
        }
    }

    pub fn into_raw(mut self) -> *mut Packet {
        let s_ptr = &mut self as *mut Packet;
        std::mem::forget(self); //this will leak memory, you will need to eventually call Packet::from_raw to release it
        s_ptr
    }

    pub fn from_raw(p: *mut Packet) -> Option<Packet> {
        let opt_packet_ref = unsafe { p.as_mut() };
        opt_packet_ref.map(|r| {
            let init_packet = std::mem::replace(r, unsafe { std::mem::uninitialized() });
            std::mem::forget(r);
            init_packet
        })
    }
}

impl Default for Packet {
    fn default() -> Self {
        Packet {
            ts: std::time::SystemTime::UNIX_EPOCH,
            data: IpcSharedMemory::from_byte(0, 0),
        }
    }
}

unsafe impl Send for Packet {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_into_raw() {
        let packet = Packet::new(std::time::UNIX_EPOCH, vec![0u8, 1u8, 2u8, 3u8]);
        let raw = packet.into_raw();
        let rt = Packet::from_raw(raw).expect("No packet");

        assert_eq!(rt.data(), &[0u8, 1u8, 2u8, 3u8]);
    }
}
