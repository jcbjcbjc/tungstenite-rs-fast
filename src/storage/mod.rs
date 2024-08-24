
use std::io::{Error, ErrorKind};
use byteorder::ByteOrder;

use self::ring_buffer::RingBuffer;

///
pub mod ring_buffer;

/// A trait for setting a value to a known state.
///
/// In-place analog of Default.
/// 
pub trait Resettable {
    ///
    fn reset(&mut self);
}

/// Error returned when enqueuing into a full buffer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Full;

/// Error returned when dequeuing from an empty buffer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Empty;

#[derive(Debug)]
pub struct U8RingBuffer<'a>(pub RingBuffer<'a,u8>);


impl<'a> U8RingBuffer<'a> {

    pub fn new() -> Self{
        Self{
            0:RingBuffer::new(vec![0;65536])
        }
    }

    pub fn read_uint<B>(&mut self, nbytes: usize) -> Result<u64,Error>
    where
        B: ByteOrder,
    {
        if self.0.len() >= nbytes {
            let value = self.0.dequeue_many_contiguous(nbytes) ;
            let int_value = match nbytes {
                2 => B::read_u16(value) as u64,
                3 => B::read_u24(value) as u64,
                4 => B::read_u32(value) as u64,
                6 => B::read_u48(value),
                8 => B::read_u64(value),
                _ => return Err(Error::new(ErrorKind::InvalidInput, "Invalid nbytes")),
            };
            Ok(int_value)
        } else {
            Err(Error::new(ErrorKind::UnexpectedEof, "Not enough data"))
        }
    }
}