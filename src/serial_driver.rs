use bytes::{BytesMut, BufMut};
use std::{ str, io, error::Error };
use tokio_util::codec::{Decoder, Encoder};
use async_trait::async_trait;
use futures::{ SinkExt, StreamExt };

#[derive(PartialEq, Clone, Debug)]
pub struct LssCommand {
    message: String,
}

impl LssCommand {
    pub fn with_param(id: u8, cmd: &str, val: i32) -> LssCommand {
        LssCommand {
            message: format!("#{}{}{}\r", id, cmd, val),
        }
    }

    pub fn simple(id: u8, cmd: &str) -> LssCommand {
        LssCommand {
            message: format!("#{}{}\r", id, cmd),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.message.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        &self.message
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct LssResponse {
    message: String
}

impl LssResponse {
    pub fn new(message: String) -> LssResponse {
        LssResponse {
            message
        }
    }

    pub fn separate(&self, separator: &str) -> Result<(u8, i32), Box<dyn Error>> {
        let len = self.message.len();
        let mut split = self
            .message[1..len-1]
            .split(separator);
        let id: u8 = split.next().ok_or("Failed to extract id")?.parse()?;
        let value: i32 = split.next().ok_or("Failed to extract value")?.parse()?;
        Ok((id, value))
    }

    pub fn separate_string(&self, separator: &str) -> Result<(u8, String), Box<dyn Error>> {
        let len = self.message.len();
        let mut split = self
            .message[1..len-1]
            .split(separator);
        let id: u8 = split.next().ok_or("Failed to extract id")?.parse()?;
        let value = split.next().ok_or("Failed to extract value")?;
        Ok((id, value.to_owned()))
    }

    /// Similar to separate but doesn't parse the ID
    /// This is useful for queries that don't return ID
    /// 
    /// Such as QID
    pub fn get_val(&self, separator: &str) -> Result<i32, Box<dyn Error>> {
        let len = self.message.len();
        let split = self
            .message[1..len-1]
            .split(separator);
        let value: i32 = split.last().ok_or("Failed to extract value")?.parse()?;
        Ok(value)
    }
}

pub struct LssCodec;

impl Decoder for LssCodec {
    type Item = LssResponse;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let command_break = src.as_ref().iter().position(|b| *b == b'\r');
        if let Some(n) = command_break {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(LssResponse::new(s.to_owned()))),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<LssCommand> for LssCodec {
    type Error = io::Error;

    fn encode(&mut self, data: LssCommand, buf: &mut BytesMut) -> Result<(), io::Error> {
        let msg = data.as_bytes();
        buf.reserve(msg.len());
        buf.put(msg);
        Ok(())
    }
}

#[async_trait]
pub trait FramedDriver {
    async fn send(&mut self, command: LssCommand) -> Result<(), Box<dyn Error>>;
    async fn receive(&mut self) -> Result<LssResponse, Box<dyn Error>>;
}

const TIMEOUT: u64 = 10;

pub struct FramedSerialDriver {
    framed_port: tokio_util::codec::Framed<tokio_serial::Serial, LssCodec>,
}

impl FramedSerialDriver {
    pub fn new(port: &str) -> Result<FramedSerialDriver, Box<dyn Error>> {
        let mut settings = tokio_serial::SerialPortSettings::default();
        settings.baud_rate = 115200;
        settings.timeout = std::time::Duration::from_millis(TIMEOUT);
        let serial_port = tokio_serial::Serial::from_path(port, &settings)?;
        Ok(FramedSerialDriver {
            framed_port: LssCodec.framed(serial_port),
        })
    }

    pub fn with_baud_rate(port: &str, baud_rate: u32) -> Result<FramedSerialDriver, Box<dyn Error>> {
        let mut settings = tokio_serial::SerialPortSettings::default();
        settings.baud_rate = baud_rate;
        settings.timeout = std::time::Duration::from_millis(TIMEOUT);
        let serial_port = tokio_serial::Serial::from_path(port, &settings)?;
        Ok(FramedSerialDriver {
            framed_port: LssCodec.framed(serial_port),
        })
    }
}

#[async_trait]
impl FramedDriver for FramedSerialDriver {
    async fn send(&mut self, command: LssCommand) -> Result<(), Box<dyn Error>> {
        self.framed_port.send(command).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<LssResponse, Box<dyn Error>> {
        let response = self.framed_port.next().await.ok_or("Failed receive message")??;
        Ok(response)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn framing_returns_none() {
        let mut payload = BytesMut::from("*5QV11200".as_bytes());
        let mut codec = LssCodec {};
        let res = codec.decode(&mut payload).unwrap();
        assert_eq!(res, None);
    }

    #[test]
    fn framing_returns_twice() {
        let mut payload = BytesMut::from("*1QV1\r*2QV2\r".as_bytes());
        let mut codec = LssCodec {};
        let res = codec.decode(&mut payload).unwrap().unwrap();
        let (id, val) = res.separate("QV").unwrap();
        assert_eq!(id, 1);
        assert_eq!(val, 1);
        let res = codec.decode(&mut payload).unwrap().unwrap();
        let (id, val) = res.separate("QV").unwrap();
        assert_eq!(id, 2);
        assert_eq!(val, 2);
        let res = codec.decode(&mut payload).unwrap();
        assert_eq!(res, None);
    }

    #[test]
    fn query_voltage_gets_extracted_from_frame() {
        let mut payload = BytesMut::from("*5QV11200\r".as_bytes());
        let mut codec = LssCodec {};
        let res = codec.decode(&mut payload).unwrap().unwrap();
        let (id, val) = res.separate("QV").unwrap();
        assert_eq!(id, 5);
        assert_eq!(val, 11200);
    }

    #[test]
    fn simple_command_serializes() {
        let command = LssCommand::simple(1, "QV");
        assert_eq!(command.as_bytes(), "#1QV\r".as_bytes())
    }

    #[test]
    fn param_command_serializes() {
        let command = LssCommand::with_param(1, "D", 10);
        assert_eq!(command.as_bytes(), "#1D10\r".as_bytes())
    }

    #[test]
    fn response_splits() {
        let res = LssResponse::new("*5QF42\r".to_owned());
        let (id, val) = res.separate("QF").unwrap();
        assert_eq!(id, 5);
        assert_eq!(val, 42);
    }

    #[test]
    fn response_splits_string() {
        let res = LssResponse::new("*5QFHEH\r".to_owned());
        let (id, val) = res.separate_string("QF").unwrap();
        assert_eq!(id, 5);
        assert_eq!(val, "HEH");
    }

    #[test]
    fn response_fail_missing_val() {
        let res = LssResponse::new("*5QF\r".to_owned());
        let err = res.separate("QF");
        assert!(err.is_err());
    }

    #[test]
    fn response_fail_missing_id() {
        let res = LssResponse::new("*QF1\r".to_owned());
        let err = res.separate("QF");
        assert!(err.is_err());
    }

    #[test]
    fn response_fail_wrong_key_split() {
        let res = LssResponse::new("*1QF2\r".to_owned());
        let err = res.separate("ZA");
        assert!(err.is_err());
    }

    #[test]
    fn response_val_only() {
        let res = LssResponse::new("*QID5\r".to_owned());
        let val = res.get_val("QID").unwrap();
        assert_eq!(val, 5);
    }
}
