use std::{
    future::Future,
    io::{self, Result},
    os::unix,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct UnixStream(unix::net::UnixStream);

impl super::UnixStream for UnixStream {
    fn pipe() -> Result<(Self, Self)> {
        unix::net::UnixStream::pair().and_then(|(s1, s2)| {
            s1.set_nonblocking(true)?;
            s2.set_nonblocking(true)?;

            Ok((Self(s1), Self(s2)))
        })
    }

    fn read<'r>(&'r mut self, buf: &'r mut [u8]) -> impl Future<Output = Result<usize>> + 'r {
        Read {
            stream: &mut self.0,
            buf,
        }
    }

    fn write<'r>(&'r mut self, buf: &'r [u8]) -> impl Future<Output = Result<usize>> + 'r {
        Write {
            stream: &mut self.0,
            buf,
        }
    }
}

#[derive(Debug)]
pub struct Read<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r mut [u8],
}

impl Future for Read<'_> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io::Read;

        let this = self.get_mut();
        match this.stream.read(this.buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

#[derive(Debug)]
pub struct Write<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r [u8],
}

impl Future for Write<'_> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io::Write;

        let this = self.get_mut();
        match this.stream.write(this.buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
