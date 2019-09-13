#![allow(unused_imports)]

mod sendfile {
    use libc::{c_int, off_t, size_t};
    use std::io::Error;

    #[cfg(feature = "large-files")]
    pub const MAX_LENGTH: u64 = off_t::max_value() as u64;
    pub const MAX_CHUNK: u64 = 0x7ffff000; // according to the Linux docs, 0x7ffff000 is the maximum length for one sendfile()

    #[inline]
    pub fn try_sendfile(
        file: c_int,
        stream: c_int,
        mut offset: off_t,
        length: usize,
    ) -> Result<off_t, (Error, off_t)> {
        let inital_offset = offset;

        match unsafe { libc::sendfile(stream, file, &mut offset as *mut off_t, length as size_t) } {
            -1 => Err((Error::last_os_error(), offset - inital_offset)),
            length => Ok(length as off_t), // a negative value is only returned in error cases
        }
    }
}

use sendfile::*;

use crate::fallback;

use libc::off_t;
use std::fs::File;
use std::io::{self, Error, ErrorKind};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

#[cfg(not(feature = "large-files"))]
pub fn send_file(file: &mut File, stream: &mut TcpStream) -> io::Result<()> {
    let length = file.metadata()?.len();

    if length == 0 {
        return Ok(());
    };

    let length = length as off_t;

    let mut offset: off_t = 0;

    while offset < length {
        let sent = match try_sendfile(
            file.as_raw_fd(),
            stream.as_raw_fd(),
            offset,
            (length - offset) as usize,
        ) {
            Ok(sent) => sent,
            Err((ref e, ref sent)) if check_error(e.kind()) => *sent,
            Err(e) => return Err(e.0),
        };

        offset += sent;
    }

    Ok(())
}

#[cfg(feature = "large-files")]
pub fn send_file(file: &mut File, stream: &mut TcpStream) -> io::Result<()> {
    let length = file.metadata()?.len();

    if length == 0 {
        return Ok(());
    };

    let mut remaining = 0;

    let length = if length > MAX_LENGTH {
        remaining = length - MAX_LENGTH; // bytes that exceed sendfile()'s capacity
        MAX_LENGTH as off_t
    } else {
        length as off_t
    };

    let mut offset: off_t = 0;

    while offset < length {
        let sent = match try_sendfile(
            file.as_raw_fd(),
            stream.as_raw_fd(),
            offset,
            (length - offset) as usize,
        ) {
            Ok(sent) => sent,
            Err((ref e, sent)) if check_error(e.kind()) => sent,
            Err(e) => return Err(e.0),
        };

        offset += sent;
    }

    if remaining != 0 {
        fallback::copy_to_end(file, stream, MAX_LENGTH)?;
    }

    Ok(())
}

#[inline]
fn check_error(e: ErrorKind) -> bool {
    e == ErrorKind::WouldBlock || e == ErrorKind::Interrupted
}

pub fn send_exact(
    file: &mut File,
    stream: &mut TcpStream,
    length: u64,
    offset: u64,
) -> io::Result<u64> {
    #[cfg(feature = "large-files")]
    {
        if offset > off_t::max_value() as u64 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "offset exceeds maximum size",
            ));
        };
    }

    let length = if length > MAX_CHUNK {
        MAX_CHUNK
    } else {
        length
    };

    match try_sendfile(
        file.as_raw_fd(),
        stream.as_raw_fd(),
        offset as off_t,
        length as usize,
    ) {
        Ok(length) => Ok(length as u64),
        Err(e) => Err(e.0),
    }
}
