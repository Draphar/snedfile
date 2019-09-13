#![allow(unused_imports)]

mod sendfile {
    use libc::{c_int, off_t, size_t};
    use std::io::Error;
    use std::ptr;

    #[inline]
    pub fn try_sendfile(
        file: c_int,
        stream: c_int,
        offset: off_t,
        nbytes: size_t,
    ) -> Result<(), (Error, off_t)> {
        let mut sent = 0;

        if unsafe {
            libc::sendfile(
                file,
                stream,
                offset,
                nbytes,
                ptr::null_mut(),
                &mut sent as *mut off_t,
                0,
            )
        } == -1
        {
            Err((Error::last_os_error(), sent))
        } else {
            Ok(())
        }
    }
}

use sendfile::*;

use crate::fallback;

use libc::{off_t, size_t};
use std::fs::File;
use std::io::{self, Error, ErrorKind};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

#[cfg(not(feature = "large-files"))]
pub fn send_file(file: &mut File, stream: &mut TcpStream) -> io::Result<()> {
    let mut offset: off_t = 0;

    loop {
        // loop until the file has been sent and handle WouldBlock and Interrupted errors

        match try_sendfile(file.as_raw_fd(), stream.as_raw_fd(), offset, 0) {
            Err((ref e, sent)) if check_error(e.kind()) => {
                offset += sent;
            }
            other => return other.map_err(|(e, _)| e),
        };
    }
}

#[cfg(feature = "large-files")]
pub fn send_file(file: &mut File, stream: &mut TcpStream) -> io::Result<()> {
    let mut offset: off_t = 0;

    loop {
        // loop until the file has been sent and handle WouldBlock and Interrupted errors

        match try_sendfile(file.as_raw_fd(), stream.as_raw_fd(), offset, 0) {
            Err((ref e, sent)) if check_error(e.kind()) => {
                let (new_offset, overflow) = offset.overflowing_add(sent);

                if overflow {
                    let offset = offset as u64 + sent as u64;

                    fallback::copy_to_end(file, stream, offset)?;
                } else {
                    // continue with the updated offset
                    offset = new_offset;
                }
            }
            other => return other.map_err(|(e, _)| e),
        };
    }
}

#[inline]
fn check_error(e: ErrorKind) -> bool {
    e == ErrorKind::WouldBlock || e == ErrorKind::Interrupted
}

pub fn send_exact(file: &File, stream: &TcpStream, length: u64, offset: u64) -> io::Result<u64> {
    #[cfg(feature = "large-files")]
    {
        if offset > off_t::max_value() as u64 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "offset exceeds maximum size",
            ));
        };
    }

    let length = if length > size_t::max_value() as u64 {
        size_t::max_value()
    } else {
        length as size_t
    };

    match try_sendfile(
        file.as_raw_fd(),
        stream.as_raw_fd(),
        offset as off_t,
        length,
    ) {
        Ok(()) => Ok(length as u64),
        Err((e, _)) => Err(e),
    }
}
