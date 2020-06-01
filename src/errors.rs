use thiserror::{Error as ThisError};

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("IO Error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Null pointer when dealing with ffi: {0:?}")]
    Ffi(#[from] std::ffi::NulError),
    #[error("Utf8 conversion error: {0:?}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Error during bincode: {0:?}")]
    Bincode(#[from] bincode::Error),
    #[error("Error receiving: {0:?}")]
    Recv(#[from] crossbeam_channel::RecvError),
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}
