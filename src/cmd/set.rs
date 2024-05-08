use crate::{RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor, RESP_OK};

#[derive(Debug)]
pub struct SAdd {
    key: String,
    members: Vec<RespFrame>,
}
#[derive(Debug)]
pub struct Sismember {
    key: String,
    value: String,
}

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        for v in self.members.iter() {
            if let RespFrame::BulkString(key) = v {
                if let Ok(value) = String::from_utf8(key.to_vec()) {
                    backend.hset(self.key.clone(), value, v.to_owned());
                }
            };
        }
        RESP_OK.clone()
    }
}
impl CommandExecutor for Sismember {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        match backend.hget(&self.key, &self.value) {
            Some(_) => RespFrame::BulkString(crate::BulkString("1".as_bytes().to_vec())),
            None => RespFrame::BulkString(crate::BulkString("0".as_bytes().to_vec())),
        }
    }
}
impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(key)) => Some(String::from_utf8(key.0)?),
            _ => None,
        };
        let fields = args.collect::<Vec<_>>();
        if fields.is_empty() || fields.len() < 1 {
            Err(CommandError::InvalidArgument("Invalid key".to_string()))
        } else {
            Ok(SAdd {
                key: key.unwrap(),
                members: fields,
            })
        }
    }
}
impl TryFrom<RespArray> for Sismember {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => {
                Ok(Sismember {
                    key: String::from_utf8(key.0)?,
                    value: String::from_utf8(field.0)?,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid key, field or value".to_string(),
            )),
        }
    }
}
