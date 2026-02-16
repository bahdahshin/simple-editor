use std::fmt;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct SoftBufferError;

impl fmt::Display for SoftBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("softbuffer operation failed")
    }
}

impl std::error::Error for SoftBufferError {}

pub struct Context<W> {
    _window: W,
}

impl<W> Context<W> {
    pub fn new(window: W) -> Result<Self, SoftBufferError> {
        Ok(Self { _window: window })
    }
}

pub struct Surface<C, W> {
    width: u32,
    height: u32,
    buffer: Vec<u32>,
    _context: PhantomData<C>,
    _window: W,
}

impl<C, W: Clone> Surface<C, W> {
    pub fn new(_context: &Context<W>, window: W) -> Result<Self, SoftBufferError> {
        Ok(Self {
            width: 1,
            height: 1,
            buffer: vec![0; 1],
            _context: PhantomData,
            _window: window,
        })
    }

    pub fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> Result<(), SoftBufferError> {
        self.width = width.get();
        self.height = height.get();
        self.buffer.resize((self.width * self.height) as usize, 0);
        Ok(())
    }

    pub fn buffer_mut(&mut self) -> Result<Buffer<'_>, SoftBufferError> {
        Ok(Buffer {
            data: &mut self.buffer,
        })
    }
}

pub struct Buffer<'a> {
    data: &'a mut [u32],
}

impl Buffer<'_> {
    pub fn present(self) -> Result<(), SoftBufferError> {
        Ok(())
    }
}

impl Deref for Buffer<'_> {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl DerefMut for Buffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}
