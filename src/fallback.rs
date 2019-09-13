#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs::File;
use std::io::{self, BufReader, ErrorKind, Read, Seek, SeekFrom, Write};
use std::net::TcpStream;

pub fn send_file(file: &mut File, stream: &mut TcpStream) -> io::Result<()> {
    let length = file.metadata()?.len();

    if length == 0 {
        return Ok(());
    };

    send_file_imp(file, stream, length)
}

pub fn send_exact(
    file: &mut File,
    stream: &mut TcpStream,
    length: u64,
    offset: u64,
) -> io::Result<u64> {
    file.seek(SeekFrom::Start(offset))?;
    io::copy(&mut file.take(length), stream)
}

#[cfg(not(any(feature = "fallback-bufreader", feature = "fallback-buf")))]
pub fn send_file_imp(file: &mut File, stream: &mut TcpStream, length: u64) -> io::Result<()> {
    let mut sent = io::copy(file, stream)?;

    while sent < length {
        sent += io::copy(file, stream)?;
    }

    Ok(())
}

#[cfg(feature = "fallback-bufreader")]
pub fn send_file_imp(file: &mut File, stream: &mut TcpStream, length: u64) -> io::Result<()> {
    let mut reader = BufReader::new(file);

    let mut sent = io::copy(&mut reader, stream)?;

    while sent < length {
        sent += io::copy(&mut reader, stream)?;
    }

    Ok(())
}

#[cfg(all(feature = "fallback-buf", not(feature = "large-files")))]
pub fn send_file_imp(file: &mut File, stream: &mut TcpStream, length: u64) -> io::Result<()> {
    let mut buf = Vec::with_capacity(length as usize);

    file.read_to_end(&mut buf)?;
    stream.write_all(&buf)?;

    Ok(())
}

#[cfg(all(feature = "fallback-buf", feature = "large-files"))]
pub fn send_file_imp(file: &mut File, stream: &mut TcpStream, length: u64) -> io::Result<()> {
    let mut remaining = length.checked_sub(usize::max_value() as u64).unwrap_or(0);

    let mut buf = Vec::with_capacity(length as usize);

    file.read_to_end(&mut buf)?;
    stream.write_all(&buf)?;

    while remaining > 0 {
        remaining -= io::copy(file, stream)?;
    }

    Ok(())
}

pub fn copy_to_end(file: &mut File, stream: &mut TcpStream, offset: u64) -> io::Result<()> {
    file.seek(SeekFrom::Start(offset))?;

    loop {
        match io::copy(file, stream) {
            Ok(0) => return Ok(()),
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    return Err(e);
                }
            }
            _ => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek, SeekFrom, Write};

    #[test]
    fn send_file_imp() {
        let mut file = tempfile::tempfile().unwrap();
        let (mut a, mut b) = tcp_test::channel();
        let data = b"dR#QaIw,";

        file.write_all(data).unwrap();

        file.seek(SeekFrom::Start(0)).unwrap();

        super::send_file_imp(&mut file, &mut a, 8).unwrap();

        let mut buf = [0; 8];
        b.read_exact(&mut buf).unwrap();
        assert_eq!(data, &buf);
    }

    #[test]
    fn send_file_exact() {
        let mut file = tempfile::tempfile().unwrap();
        let (mut a, mut b) = tcp_test::channel();
        let data = b"dR#QaIw,";

        file.write_all(data).unwrap();

        super::send_exact(&mut file, &mut a, 8, 1).unwrap();

        let mut buf = [0; 7];
        b.read_exact(&mut buf).unwrap();
        assert_eq!(&data[1..], &buf);
    }

    #[test]
    fn copy_to_end() {
        let mut file = tempfile::tempfile().unwrap();
        let (mut a, mut b) = tcp_test::channel();
        let data = b"Ht9!Kwk}";

        file.write_all(data).unwrap();

        super::copy_to_end(&mut file, &mut a, 3).unwrap();

        let mut buf = [0; 5];
        b.read_exact(&mut buf).unwrap();
        assert_eq!(&data[3..], &buf);
    }
}
