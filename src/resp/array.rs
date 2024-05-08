use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{BulkString, RespDecode, RespEncode, RespError, RespFrame};

use super::{calc_total_length, parse_length, BUF_CAP, CRLF_LEN, NULL_ARRAY};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>, bool);

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);

        if self.1 {
            buf.extend_from_slice(b"*-1\r\n");
            return buf;
        }
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
// null array: "*-1\r\n"
// FIXME: need to handle incomplete
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if check_null_array(buf) {
            let mut frames = Vec::with_capacity(1);
            let frame: RespFrame = BulkString::new(b"-1\r\n").into();
            frames.push(frame);
            return Ok(RespArray::new(frames, true));
        }
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(RespArray::new(frames, false))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        if check_null_array(buf) {
            return Ok(4);
        }
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

fn check_null_array(buf: &[u8]) -> bool {
    if buf.starts_with(NULL_ARRAY) {
       return true
    }
    false
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>, is_null_flag: bool) -> Self {
        RespArray(s.into(), is_null_flag)
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BulkString;
    use anyhow::Result;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray::new(
            vec![
                BulkString::new("set".to_string()).into(),
                BulkString::new("hello".to_string()).into(),
                BulkString::new("world".to_string()).into(),
            ],
            false,
        )
        .into();
        assert_eq!(
            &frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_null_array_encode() {
        let frame: RespFrame =
            RespArray::new(vec![BulkString::new("nil".to_string()).into()], true).into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_null_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"-1\r\n".into()], true));

        Ok(())
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new([b"set".into(), b"hello".into()], false)
        );

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new([b"set".into(), b"hello".into()], false)
        );

        Ok(())
    }
}
