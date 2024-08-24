//! A buffer for reading data from the network.
//!
//! The `ReadBuffer` is a buffer of bytes similar to a first-in, first-out queue.
//! It is filled by reading from a stream supporting `Read` and is then
//! accessible as a cursor for reading bytes.

use std::{io::{Cursor, Read, Result as IoResult}};

use bytes::Buf;

/// A FIFO buffer for reading packets from the network.
// #[derive(Debug)]
// pub struct ReadCycledBuffer<const CHUNK_SIZE: usize>{
//     storage: Box<[u8; CHUNK_SIZE]>,
//     read_ptr: usize,
//     write_ptr: usize,
// }

// // impl<const CHUNK_SIZE: usize> Read for ReadCycledBuffer<CHUNK_SIZE>{
// //     fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        
// //     }
// // }

// impl<const CHUNK_SIZE: usize> ReadCycledBuffer<CHUNK_SIZE> {
//     /// 创建新的环形缓冲区
//     pub fn new() -> Self {
//         ReadCycledBuffer {
//             read_ptr: 0,
//             write_ptr: 0,
//             storage: Box::new([0; CHUNK_SIZE]),
//         }
//     }

//     /// 获得读的位置
//     pub fn position(&self) -> usize{
//         self.read_ptr
//     }

//     /// 设置读的位置
//     pub fn set_position(&mut self,position:usize){
//         self.read_ptr = position
//     }

// /// 读取指定大小的数据
// use std::slice;

// /// 读取指定大小的数据
// pub fn read(&mut self, size: usize) -> Result<&mut [u8], &'static str> {
//     let available_data = self.available_data();
//     if size > available_data {
//         return Err("Not enough data in the buffer");
//     }

//     // 计算读取范围的开始和结束位置
//     let start = self.read_ptr;
//     let end = (self.read_ptr + size) % CHUNK_SIZE;

//     // 创建一个空的可变切片
//     let data_slice: &mut [u8];

//     if start <= end {
//         // 直接获取切片的指针和长度
//         let ptr = &mut self.storage[start] as *mut u8;
//         let len = end - start;
//         data_slice = unsafe {
//             slice::from_raw_parts_mut(ptr, len)
//         };
//     } else {
//         let (first_part, second_part) = self.storage.split_at_mut(end);
//         let (second_part, _) = second_part.split_at_mut(size - (end - start));

//         // 合并 first_part 和 second_part 的切片
//         data_slice = &mut [][..]; // 创建一个空切片
//         data_slice = &mut data_slice[..0]; // 清空 data_slice，使其为空
//         data_slice = &mut data_slice.join_mut(first_part);
//         data_slice = &mut data_slice.join_mut(second_part);
//     }

//     // 更新 read_ptr
//     self.read_ptr = end;

//     Ok(data_slice)
// }


    


//     /// 返回可用空间大小
//     pub fn available_space(&self) -> usize {
//         (self.read_ptr + CHUNK_SIZE - self.write_ptr) % CHUNK_SIZE
//     }

//     /// 返回可用数据大小
//     pub fn available_data(&self) -> usize {
//         (self.write_ptr + CHUNK_SIZE - self.read_ptr) % CHUNK_SIZE
//     }
    
//     /// 获得可变切片
//     pub fn get_slice(&mut self) -> &mut [u8] {
//         let available_space = self.available_space();
//         let start = self.write_ptr;
//         let end = (self.write_ptr + available_space) % CHUNK_SIZE;

//         if start <= end {
//             &mut self.storage[start..end]
//         } else {
//             let (first_part, second_part) = self.storage.split_at_mut(end);
//             let (_, second_part) = second_part.split_at_mut(available_space - (end - start));

//             // 将 first_part 和 second_part 组合成一个可变切片
//             let combined_slice_len = first_part.len() + second_part.len();
//             let combined_slice_ptr = first_part.as_mut_ptr();
//             unsafe {
//                 slice::from_raw_parts_mut(combined_slice_ptr, combined_slice_len)
//             }
//         }
//     }

    

//     /// Read next portion of data from the given input stream.
//     pub fn read_from<S: Read>(&mut self, stream: &mut S) -> IoResult<usize> {
//        // self.clean_up();
//         let size = stream.read(&mut *self.get_slice())?;
//         self.write_ptr = (self.write_ptr + size) % CHUNK_SIZE;
//         Ok(size)
//     }

//      /// Create a input buffer filled with previously read data.
//      pub fn from_partially_read(part: Vec<u8>) -> Self {
//         Self { storage: Box::new([0;CHUNK_SIZE]), read_ptr: 0, write_ptr: part.len() }
//     }
// }

/// A FIFO buffer for reading packets from the network.
#[derive(Debug)]
pub struct ReadBuffer<const CHUNK_SIZE: usize> {
    storage: Cursor<Vec<u8>>,
    chunk: Box<[u8; CHUNK_SIZE]>,
}

impl<const CHUNK_SIZE: usize> ReadBuffer<CHUNK_SIZE> {
    /// Create a new empty input buffer.
    pub fn new() -> Self {
        Self::with_capacity(CHUNK_SIZE)
    }

    /// Create a new empty input buffer with a given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_partially_read(Vec::with_capacity(capacity))
    }

    /// Create a input buffer filled with previously read data.
    pub fn from_partially_read(part: Vec<u8>) -> Self {
        Self { storage: Cursor::new(part), chunk: Box::new([0; CHUNK_SIZE]) }
    }

    /// Get a cursor to the data storage.
    pub fn as_cursor(&self) -> &Cursor<Vec<u8>> {
        &self.storage
    }

    /// Get a cursor to the mutable data storage.
    pub fn as_cursor_mut(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.storage
    }

    /// Consume the `ReadBuffer` and get the internal storage.
    pub fn into_vec(mut self) -> Vec<u8> {
        // Current implementation of `tungstenite-rs` expects that the `into_vec()` drains
        // the data from the container that has already been read by the cursor.
        self.clean_up();

        // Now we can safely return the internal container.
        self.storage.into_inner()
    }

    /// Read next portion of data from the given input stream.
    pub fn read_from<S: Read>(&mut self, stream: &mut S) -> IoResult<usize> {

        self.clean_up();
        let size = stream.read(&mut *self.chunk)?;
        self.storage.get_mut().extend_from_slice(&self.chunk[..size]);
        Ok(size)
    }

    /// Cleans ups the part of the vector that has been already read by the cursor.
    fn clean_up(&mut self) {
        let pos = self.storage.position() as usize;
        self.storage.get_mut().drain(0..pos).count();
        self.storage.set_position(0);
    }
}

impl<const CHUNK_SIZE: usize> Buf for ReadBuffer<CHUNK_SIZE> {
    fn remaining(&self) -> usize {
        Buf::remaining(self.as_cursor())
    }

    fn chunk(&self) -> &[u8] {
        Buf::chunk(self.as_cursor())
    }

    fn advance(&mut self, cnt: usize) {
        Buf::advance(self.as_cursor_mut(), cnt)
    }
}

impl<const CHUNK_SIZE: usize> Default for ReadBuffer<CHUNK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_reading() {
        let mut input = Cursor::new(b"Hello World!".to_vec());
        let mut buffer = ReadBuffer::<4096>::new();
        let size = buffer.read_from(&mut input).unwrap();
        assert_eq!(size, 12);
        assert_eq!(buffer.chunk(), b"Hello World!");
    }

    #[test]
    fn reading_in_chunks() {
        let mut inp = Cursor::new(b"Hello World!".to_vec());
        let mut buf = ReadBuffer::<4>::new();

        let size = buf.read_from(&mut inp).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf.chunk(), b"Hell");

        buf.advance(2);
        assert_eq!(buf.chunk(), b"ll");
        assert_eq!(buf.storage.get_mut(), b"Hell");

        let size = buf.read_from(&mut inp).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf.chunk(), b"llo Wo");
        assert_eq!(buf.storage.get_mut(), b"llo Wo");

        let size = buf.read_from(&mut inp).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf.chunk(), b"llo World!");
    }
}
