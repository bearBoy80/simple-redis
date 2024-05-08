use super::{parse_length, CRLF_LEN, NULL_BULK_STRING};
use crate::{RespDecode, RespEncode, RespError};
use bytes::{Buf, BytesMut};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);

// - bulk string: "$<length>\r\n<data>\r\n" or - null bulk string: "$-1\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        if NULL_BULK_STRING.eq_ignore_ascii_case(&self) {
            buf.extend_from_slice(&format!("${}\r\n", "-1").into_bytes());
        } else {
            buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
            buf.extend_from_slice(&self);
            buf.extend_from_slice(b"\r\n");
        }
        buf
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        //if buf is null bulk string
        if check_null_bulk_string(buf) {
            return Ok(BulkString::new(NULL_BULK_STRING.to_vec()));
        }
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        if check_null_bulk_string(buf) {
            Ok(4)
        } else {
            let (end, len) = parse_length(buf, Self::PREFIX)?;
            Ok(end + CRLF_LEN + len + CRLF_LEN)
        }
    }
}

fn check_null_bulk_string(buf: &[u8]) -> bool {
    if buf.starts_with(NULL_BULK_STRING) {
        return true;
    }
    false
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<String> for BulkString {
    fn from(s: String) -> Self {
        BulkString(s.into_bytes())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"hello".to_vec()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"$-1\r\n".to_vec()).into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        Ok(())
    }

    #[test]
    fn test_null_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"$-1\r\n"));

        Ok(())
    }
}
