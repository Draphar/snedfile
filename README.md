# snedfile - Rust cross-platform sendfile() abstractions

[![travis-badge]][travis]
[![appveyor-badge]][appveyor]
[![crates.io-badge]][crates.io]
[![docs-badge]][docs]
[![license-badge]][license]

[travis-badge]: https://travis-ci.com/Draphar/snedfile.svg?branch=master
[travis]: https://travis-ci.com/Draphar/snedfile
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/github/Draphar/snedfile?svg=true&branch=master
[appveyor]: https://ci.appveyor.com/project/Draphar/snedfile
[crates.io-badge]: https://img.shields.io/crates/v/snedfile.svg
[crates.io]: https://crates.io/crates/snedfile
[docs-badge]: https://docs.rs/snedfile/badge.svg
[docs]: https://docs.rs/snedfile
[license-badge]: https://img.shields.io/crates/l/snedfile.svg
[license]: https://github.com/Draphar/snedfile/blob/master/LICENSE

Natively supported using `sendfile()` are Linux, Android, MacOS, iOS, FreeBSD and DragonFlyBSD,
and every other `std`-platform using a fallback.

# Usage

This library is designed to make transmitting files as easy as possible.
If you have a file and a TCP stream, all you have to do is

```rust
use snedfile::send_file;

fn transmit(path: impl AsRef<Path>, stream: TcpStream) -> io::Result<()> {
    let file = File::open(path)?;

    send_file(&mut file, &mut stream)
}
```

Trivial errors as well as optimally using the native system capabilities are handled by the implementation.

Alternatively, there is a more low-level solution:

```rust
use snedfile::send_exact;

fn transmit(path: impl AsRef<Path>, stream: TcpStream) -> io::Result<()> {
    let file = File::open(path)?;

    send_exact(&mut file, &mut stream, file.metadata()?.len(), 0)
}
```
