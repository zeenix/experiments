use std::{future::Future, io::Result};

pub mod naive;

pub trait UnixStream: Sized {
    fn pipe() -> Result<(Self, Self)>;

    fn read<'r>(&'r mut self, buf: &'r mut [u8]) -> impl Future<Output = Result<usize>> + 'r;

    fn write<'r>(&'r mut self, buf: &'r [u8]) -> impl Future<Output = Result<usize>> + 'r;
}
