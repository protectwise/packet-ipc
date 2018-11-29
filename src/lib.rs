#[macro_use] extern crate error_chain;

pub mod errors {
    use std;

    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Io(std::io::Error) #[doc = "Error during IO"];
            Ffi(std::ffi::NulError) #[doc = "Error during FFI conversion"];
            Utf8(std::str::Utf8Error) #[doc = "Error during UTF8 conversion"];
            ChannelCancelled(futures::sync::oneshot::Canceled) #[doc = "Channel was cancelled"];
            ChannelReceive(crossbeam_channel::RecvError) #[doc = "Error during receive"];
        }
        errors {
            ChannelError {
                display("Channel encountered an error")
            }
            ConnectionFailed {
                display("Failed to form connection")
            }
            IpcFailure {
                display("IPC Failure")
            }
            Bincode {
                display("Bincode failure")
            }
        }
    }
}

pub mod client;
pub mod packet;
pub mod server;