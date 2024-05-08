use super::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::{RespArray, RespFrame};
use anyhow::Result;
#[derive(Debug)]
pub struct Echo {
    msg: String,
}

impl CommandExecutor for Echo {
    fn execute(self, _backend: &crate::Backend) -> RespFrame {
        let bulk_string = crate::BulkString::new(self.msg);
        RespFrame::BulkString(bulk_string)
    }
}
impl TryFrom<RespArray> for Echo {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Echo {
                msg: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid message".to_string())),
        }
    }
}
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::{
        cmd::{echo::Echo, CommandExecutor},
        Backend, RespArray, RespDecode, RespFrame,
    };

    #[test]
    fn test_echo_from_resparry() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: Echo = frame.try_into()?;
        assert_eq!(result.msg, "hello");

        Ok(())
    }
    #[test]
    fn test_echo_hello() -> Result<()> {
        let backend = Backend::new();
        let cmd = Echo {
            msg: "hello".into(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"hello".into()));
        Ok(())
    }
}
