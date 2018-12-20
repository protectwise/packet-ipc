use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO Error")]
    Io(#[fail(cause)] std::io::Error),
    #[fail(display = "Null pointer when dealing with ffi")]
    Ffi(#[fail(cause)] std::ffi::NulError),
    #[fail(display = "Utf8 conversion error")]
    Utf8(#[fail(cause)] std::str::Utf8Error),
    #[fail(display = "Error during bincode")]
    Bincode(#[fail(cause)] bincode::Error),
    #[fail(display = "Error receiving")]
    Recv(#[fail(cause)] crossbeam_channel::RecvError),
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}
