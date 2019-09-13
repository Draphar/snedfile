/*!
Cross-platform abstractions for the `sendfile()` system call.
Natively supported are Linux, android, MacOS, iOS, FreeBSD and DragonFlyBSD,
and every other `std`-platform using a fallback.

# Implementations

All native implementation support a maximum file length of [`off_t::max_value()`].

All implementations handle `WouldBlock` and `Interrupted` errors.

## Linux and android

The [`sendfile(2)`][linux] system call is used.

The file length is required and the functions fails if `file.metadata()` fails.

## MacOS and optionally iOS

The [`sendfile(2)`][macos] system call is used.

The file length is only required after the maximum file length has been sent.
Note that there are sparse reports of `sendfile()` [being buggy on iOS],
so if you prefer to use the fallback for `target_os = "ios"`,
disable the `ios-sendfile` feature which is enabled by default.

## FreeBSD and DragonFlyBSD

The [`sendfile(2)`][bsd] system call is used.

The file length is not required.

## Fallback

There are two features to change the fallback behavior:

The `fallback-bufreader` feature is enabled by default.
It sends the file using [`io::copy()`] after wrapping it in a [`BufReader`].
`Interrupted` errors are handled by the implementation.

If the `fallback-buf` feature is enabled,
the entire contents of the file are loaded into a `Vec` first,
which is then written to the stream at once.
Only the first `usize::max_value()` may be transmitted.

If both features are disabled the file is transmitted by repeatedly using bare [`io::copy()`] until all bytes have been sent.

# Large files

If you expected to send files larger than 2 gigabytes from a 32-bit system or
files larger than 8192 *peta*bytes from a 64-bit system,
enable the `large-files` feature which supports all file sizes up to `u64::max_value()`,
and if the files become to large for the native solutions a fallback is used.

[linux]: http://man7.org/linux/man-pages/man2/sendfile.2.html
[macos]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendfile.2.html
[bsd]: https://www.freebsd.org/cgi/man.cgi?query=sendfile
[being buggy on iOS]: https://blog.phusion.nl/2015/06/04/the-brokenness-of-the-sendfile-system-call/
[`off_t::max_value()`]: https://docs.rs/libc/0.2/libc/type.off_t.html
[`io::copy()`]: https://doc.rust-lang.org/stable/std/io/fn.copy.html
[`BufReader`]: https://doc.rust-lang.org/stable/std/io/struct.BufReader.html
*/

#![deny(missing_docs)]

//todo: Remove libc

extern crate libc;
#[cfg(test)]
extern crate tempfile;

#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "linux.rs"]
mod imp;

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
#[path = "freebsd.rs"]
mod imp;

#[cfg(any(target_os = "macos", all(target_os = "ios", feature = "ios-sendfile")))]
#[path = "macos.rs"]
mod imp;

mod fallback;

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    all(target_os = "ios", feature = "ios-sendfile"),
    target_os = "freebsd",
    target_os = "dragonfly"
)))]
use fallback as imp;

#[cfg(all(feature = "fallback-bufreader", feature = "fallback-buf"))]
compile_error!("Only one `fallback-*` feature can enabled");

use std::fs::File;
use std::io;
use std::net::TcpStream;

/// Sends the entire contents of a file to a TCP stream.
///
/// The file must be opened for reading.
///
/// Depending on the backend, a more efficient method is used which prevents copying all data to userspace.
/// See the [module documentation] for more.
///
/// This function is optimized in a way that it only needs to be called once on a file and stream.
/// Trivial errors are handled and the native `sendfile()` is used as much as possible.
///
/// If the file has a length of `0`, this function returns successfully without doing additional work.
///
/// This function does not guarantee respecting the file offset, if it already has been changed by using `Seek` or `Read`.
///
/// # Example
///
/// ```
/// use snedfile::send_file;
/// # use std::io;
/// # use std::fs::File;
/// # use std::net::TcpStream;
///
/// // somewhere in a server for static files
/// fn serve_static(file: &mut File, stream: &mut TcpStream) -> io::Result<()> {
///     send_file(file, stream)
/// }
/// ```
///
/// [module documentation]: index.html
#[inline]
pub fn send_file(file: &mut File, stream: &mut TcpStream) -> io::Result<()> {
    imp::send_file(file, stream)
}

/// Send a specific amount of bytes from a specific offset within a file.
///
/// The amount of bytes successfully sent is returned.
///
/// # Implementation notes
///
/// If the offset is larger than the implementation can handle,
/// an error of type `ErrorKind::InvalidData` is returned.
///
/// Exactly one system call is used, if available.
///
/// Regardless of the enabled features,
/// the fallback of `send_exact()` *always* uses the bare `io::copy()`.
///
/// The behaviour is not specified (but not undefined) if the offset goes beyond the end of the file.
///
/// No kinds of errors are handled.
///
/// # Example
///
/// ```
/// use snedfile::send_exact;
/// # use std::io;
/// # use std::fs::File;
/// # use std::net::TcpStream;
///
/// fn try_serve_static(mut file: File, mut stream: TcpStream) -> io::Result<u64> {
///     let len = file.metadata()?.len();
///
///     // the same as the example from `send_file`,
///     // but with less automatic error handling
///     send_exact(&mut file, &mut stream, len, 0)
/// }
/// ```
#[inline]
pub fn send_exact(
    file: &mut File,
    stream: &mut TcpStream,
    bytes: u64,
    offset: u64,
) -> io::Result<u64> {
    imp::send_exact(file, stream, bytes, offset)
}
