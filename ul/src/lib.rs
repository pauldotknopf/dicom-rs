#[cfg(test)]
#[macro_use]
extern crate matches;

use std::io;
use std::net::{Shutdown, TcpStream};

/// The recommended port number (104) to use, but requires admin access.
pub const DEFAULT_PORT_PRIMARY: u16 = 104;
/// If no admin access, then use this port number (11112).
pub const DEFAULT_PORT_SECONDARY: u16 = 11112;

pub mod asso;
pub mod dimse;
pub mod error;
pub mod fsm;
pub mod pdu;

pub enum NetStream {
    #[cfg(test)]
    Mocked(mockstream::SharedMockStream),
    Tcp(TcpStream),
}

impl NetStream {
    pub fn shutdown(&mut self, how: Shutdown) -> io::Result<()> {
        match *self {
            #[cfg(test)]
            NetStream::Mocked(ref mut _s) => Ok(()),
            NetStream::Tcp(ref mut s) => s.shutdown(how),
        }
    }
}

impl std::io::Read for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match *self {
            #[cfg(test)]
            NetStream::Mocked(ref mut s) => s.read(buf),
            NetStream::Tcp(ref mut s) => s.read(buf),
        }
    }
}

impl std::io::Write for NetStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match *self {
            #[cfg(test)]
            NetStream::Mocked(ref mut s) => s.write(buf),
            NetStream::Tcp(ref mut s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match *self {
            #[cfg(test)]
            NetStream::Mocked(ref mut s) => s.flush(),
            NetStream::Tcp(ref mut s) => s.flush(),
        }
    }
}
