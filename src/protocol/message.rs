
use std::{
    convert::{ From, Into, TryFrom},
    fmt,
    result::Result as StdResult,
    str, io::Read
};

use super::frame::{CloseFrame, ReadFrame,WriteFrame};
use crate::error::{ Error, Result};

#[derive(Debug)]
pub struct BitCollector{
    data:Vec<u8>
}

impl BitCollector{
    pub fn new() -> BitCollector{    
        BitCollector { data: Vec::with_capacity(500)}
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }
    #[inline(always)]
    pub fn extend(&mut self, tail: &mut ReadFrame) -> Result<()> {
        let payload :&[u8] = tail.payload();
        self.data.extend(payload);
        Ok(())
    }
    #[inline(always)]
    pub fn reuse(&mut self){
        self.data.clear()
    }
    pub fn read(&self,buf: &mut[u8])->std::io::Result<usize> {
        let len = self.data.len();
        if buf.len() < len{
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Buffer is too small"));
        }
        buf[..len].copy_from_slice(self.data.as_slice());
        Ok(len)    
    }
    pub fn into_vec(&self)-> Vec<u8> {
        self.data.clone()
    }
    pub fn into_string(&self) -> Result<String> {
        Ok(String::from_utf8(self.data.clone())?)
    }
}

/// A struct representing the incomplete message.
#[derive(Debug)]
pub struct IncompleteMessage {
    msg_type : IncompleteMessageType,
    collector: BitCollector
}

impl IncompleteMessage {
    /// Create new.
    pub fn new() -> Self {
        IncompleteMessage {
            collector:BitCollector::new(),
            msg_type: IncompleteMessageType::Text,
        }
    }

    /// Create new.
    pub fn reuse(&mut self,msg_type:IncompleteMessageType) ->&mut IncompleteMessage{
        self.msg_type = msg_type;
        self.collector.reuse();
        self
    }
    
    // /// Create new.
    // pub fn new(msg_type:IncompleteMessageType) -> Self {
    //     IncompleteMessage {
    //         collector:BitCollector::new(),
    //         msg_type,
    //     }
    // }

    /// Get the current filled size of the buffer.
    pub fn len(&self) -> usize {
        self.collector.len()
    }

    /// Add more data to an existing message.
    pub fn extend(&mut self, tail: &mut ReadFrame) -> Result<()> {
        // Always have a max size. This ensures an error in case of concatenating two buffers
        // of more than `usize::max_value()` bytes in total.
        // let max_size = size_limit.unwrap_or_else(usize::max_value);
        // let my_size = self.len();
        // let portion_size = tail.as_ref().len();
        // // Be careful about integer overflows here.
        // if my_size > max_size || portion_size > max_size - my_size {
        //     return Err(Error::Capacity(CapacityError::MessageTooLong {
        //         size: my_size + portion_size,
        //         max_size,
        //     }));
        // }
        self.collector.extend(tail)
    }

    /// Convert an incomplete message into a complete one.
    pub fn complete(&mut self) -> Result<Message> {
        match self.msg_type {
            IncompleteMessageType::Binary => {
                let non_null_bit_collector = &mut self.collector;
                Ok(Message::ReadBinary(non_null_bit_collector))
            }
            IncompleteMessageType::Text => {
                let non_null_bit_collector = &mut self.collector;
                Ok(Message::ReadText(non_null_bit_collector))
            }
        }
    }
}

/// The type of incomplete message.
#[derive(Debug)]
pub enum IncompleteMessageType {
    Text,
    Binary,
}

/// An enum representing the various forms of a WebSocket message.
#[derive(Debug)]
pub enum Message {
    /// A text WebSocket message
    Text(String),
    /// A binary WebSocket message
    Binary(Vec<u8>),
    /// A text WebSocket message
    ReadText(*mut BitCollector),
    /// A binary WebSocket message
    ReadBinary(*mut BitCollector),
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Ping(Vec<u8>),
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Pong(Vec<u8>),
    /// A close message with the optional close frame.
    Close(Option<CloseFrame<'static>>),
    /// Raw frame. Note, that you're not going to get this value while reading the message.
    ReadFrame(ReadFrame),
    /// Raw frame. Note, that you're not going to get this value while reading the message.
    WriteFrame(WriteFrame),
}

impl Read for Message{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>{
        match self{
            Message::Text(_) => todo!(),
            Message::Binary(_) =>  todo!(),
            Message::ReadText(frame) => {
               unsafe{
                  (*(*frame)).read(buf)
               }
            },
            Message::ReadBinary(frame) => {
                unsafe{
                    (*(*frame)).read(buf)
                }
            },
            Message::Ping(_) => todo!(),
            Message::Pong(_) => todo!(),
            Message::Close(_) => todo!(),
            Message::ReadFrame(_) => todo!(),
            Message::WriteFrame(_) => todo!(),
        }
    }
}

impl Message {
    /// Create a new text WebSocket message from a stringable.
    pub fn text<S>(string: S) -> Message
    where
        S: Into<String>,
    {
        Message::Text(string.into())
    }

    /// Create a new binary WebSocket message by converting to `Vec<u8>`.
    pub fn binary<B>(bin: B) -> Message
    where
        B: Into<Vec<u8>>,
    {
        Message::Binary(bin.into())
    }

    /// Indicates whether a message is a text message.
    pub fn is_text(&self) -> bool {
        matches!(*self, Message::Text(_)|Message::ReadText(_))
    }

    /// Indicates whether a message is a binary message.
    pub fn is_binary(&self) -> bool {
        matches!(*self, Message::Binary(_)|Message::ReadBinary(_))
    }

    /// Indicates whether a message is a ping message.
    pub fn is_ping(&self) -> bool {
        matches!(*self, Message::Ping(_))
    }

    /// Indicates whether a message is a pong message.
    pub fn is_pong(&self) -> bool {
        matches!(*self, Message::Pong(_))
    }

    /// Indicates whether a message is a close message.
    pub fn is_close(&self) -> bool {
        matches!(*self, Message::Close(_))
    }

    /// Get the length of the WebSocket message.
    pub fn len(&self) -> usize {
        unsafe{
            match *self {
                Message::Text(ref string) => string.len(),
                Message::Binary(ref data) => {
                    data.len()
                }
                Message::Ping(ref data) | Message::Pong(ref data) => {
                    data.len()
                }
                Message::Close(ref data) => data.as_ref().map(|d| d.reason.len()).unwrap_or(0),
                Message::ReadFrame(ref frame) => frame.len(),
                Message::WriteFrame(ref frame) => frame.len(),
                Message::ReadText(frame) => (*frame).len(),
                Message::ReadBinary(frame) => (*frame).len(),
            }
        }
    }

    /// Returns true if the WebSocket message has no content.
    /// For example, if the other side of the connection sent an empty string.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Consume the WebSocket and return it as binary data.
    pub fn into_data(self) -> Vec<u8> {
        match self {
            Message::Text(string) => string.into_bytes(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data,
            Message::Close(None) => Vec::new(),
            Message::Close(Some(frame)) => frame.reason.into_owned().into_bytes(),
            //Message::ReadFrame(frame) => frame.into_data(),
            Message::WriteFrame(frame) => frame.into_data(),
            Message::ReadText(_) => todo!(),
            Message::ReadBinary(_) => todo!(),
            Message::ReadFrame(_) => todo!(),
        }
    }

    /// Attempt to consume the WebSocket message and convert it to a String.
    pub fn into_text(self) -> Result<String> {
        match self {
            Message::Text(string) => Ok(string),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => {
                Ok(String::from_utf8(data)?)
            }
            Message::Close(None) => Ok(String::new()),
            Message::Close(Some(frame)) => Ok(frame.reason.into_owned()),
            //Message::ReadFrame(frame) => Ok(frame.into_string()?),
            Message::WriteFrame(frame) => Ok(frame.into_string()?),
            Message::ReadText(_) => todo!(),
            Message::ReadBinary(_) => todo!(),
            Message::ReadFrame(_) => todo!(),
        }
    }

    /// Attempt to get a &str from the WebSocket message,
    /// this will try to convert binary data to utf8.
    pub fn to_text(&self) -> Result<&str> {
        match *self {
            Message::Text(ref string) => Ok(string),
            Message::Binary(ref data) | Message::Ping(ref data) | Message::Pong(ref data) => {
                Ok(str::from_utf8(data)?)
            }
            Message::Close(None) => Ok(""),
            Message::Close(Some(ref frame)) => Ok(&frame.reason),
            //Message::ReadFrame(ref frame) => Ok(frame.to_text()?),
            Message::WriteFrame(ref frame) => Ok(frame.to_text()?),
            Message::ReadText(_) => todo!(),
            Message::ReadBinary(_) => todo!(),
            Message::ReadFrame(_) => todo!(),
        }
    }
}

impl From<String> for Message {
    fn from(string: String) -> Self {
        Message::text(string)
    }
}

impl<'s> From<&'s str> for Message {
    fn from(string: &'s str) -> Self {
        Message::text(string)
    }
}

impl<'b> From<&'b [u8]> for Message {
    fn from(data: &'b [u8]) -> Self {
        Message::binary(data)
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Message::binary(data)
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        message.into_data()
    }
}

impl TryFrom<Message> for String {
    type Error = Error;

    fn try_from(value: Message) -> StdResult<Self, Self::Error> {
        value.into_text()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        if let Ok(string) = self.to_text() {
            write!(f, "{}", string)
        } else {
            write!(f, "Binary Data<length={}>", self.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let t = Message::text("test".to_owned());
        assert_eq!(t.to_string(), "test".to_owned());

        let bin = Message::binary(vec![0, 1, 3, 4, 241]);
        assert_eq!(bin.to_string(), "Binary Data<length=5>".to_owned());
    }

    #[test]
    fn binary_convert() {
        let bin = [6u8, 7, 8, 9, 10, 241];
        let msg = Message::from(&bin[..]);
        assert!(msg.is_binary());
        assert!(msg.into_text().is_err());
    }

    #[test]
    fn binary_convert_vec() {
        let bin = vec![6u8, 7, 8, 9, 10, 241];
        let msg = Message::from(bin);
        assert!(msg.is_binary());
        assert!(msg.into_text().is_err());
    }

    #[test]
    fn binary_convert_into_vec() {
        let bin = vec![6u8, 7, 8, 9, 10, 241];
        let bin_copy = bin.clone();
        let msg = Message::from(bin);
        let serialized: Vec<u8> = msg.into();
        assert_eq!(bin_copy, serialized);
    }

    #[test]
    fn text_convert() {
        let s = "kiwotsukete";
        let msg = Message::from(s);
        assert!(msg.is_text());
    }
}
