use tcp_test::*;

use snedfile::*;

use std::fs::File;
use std::io::Read;

#[test]
fn entire_file() {
    let (mut local, mut remote) = channel();

    let mut read_handle = File::open("tests/test_file").unwrap();

    send_file(&mut read_handle, &mut local).expect("send_file() failed");

    let mut buf = [0; 13];
    remote.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"Hello world!\n");
}

#[test]
fn exact() {
    let (mut local, mut remote) = channel();

    let mut read_handle = File::open("tests/test_file").unwrap();

    send_exact(&mut read_handle, &mut local, 5, 6).expect("send_file() failed");
    send_exact(&mut read_handle, &mut local, 1, 5).expect("send_file() failed");
    send_exact(&mut read_handle, &mut local, 5, 0).expect("send_file() failed");
    send_exact(&mut read_handle, &mut local, 2, 11).expect("send_file() failed");

    let mut buf = [0; 13];
    remote.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"world Hello!\n");
}
