use crate::message_types::LssDriverError;
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use std::{io, str};
#[cfg(target_family = "windows")]
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tokio_util::codec::{Decoder, Encoder};

type DriverResult<T> = Result<T, LssDriverError>;

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
    message: String,
}

impl LssResponse {
    pub fn new(message: String) -> LssResponse {
        LssResponse { message }
    }

    pub fn separate(&self, separator: &str) -> DriverResult<(u8, i32)> {
        let len = self.message.len();
        let mut split = self.message[1..len - 1].split(separator);
        let id: u8 = split
            .next()
            .ok_or_else(|| {
                LssDriverError::PacketParsingError(String::from("Failed to extract id"))
            })?
            .parse()
            .map_err(|_| LssDriverError::PacketParsingError(String::from("Failed parsing id")))?;
        let value: i32 = split
            .next()
            .ok_or_else(|| {
                LssDriverError::PacketParsingError("Failed to extract value".to_owned())
            })?
            .parse()
            .map_err(|_| {
                LssDriverError::PacketParsingError(String::from("Failed parsing value"))
            })?;
        Ok((id, value))
    }

    pub fn separate_string(&self, separator: &str) -> DriverResult<(u8, String)> {
        let len = self.message.len();
        let mut split = self.message[1..len - 1].split(separator);
        let id: u8 = split
            .next()
            .ok_or_else(|| {
                LssDriverError::PacketParsingError(String::from("Failed to extract id"))
            })?
            .parse()
            .map_err(|_| LssDriverError::PacketParsingError(String::from("Failed parsing id")))?;
        let value = split.next().ok_or_else(|| {
            LssDriverError::PacketParsingError("Failed to extract value".to_owned())
        })?;
        Ok((id, value.to_owned()))
    }

    /// Similar to separate but doesn't parse the ID
    /// This is useful for queries that don't return ID
    ///
    /// Such as QID
    pub fn get_val(&self, separator: &str) -> DriverResult<i32> {
        let len = self.message.len();
        let split = self.message[1..len - 1].split(separator);
        let value: i32 = split
            .last()
            .ok_or_else(|| {
                LssDriverError::PacketParsingError("failed to extract value".to_owned())
            })?
            .parse()
            .map_err(|_| LssDriverError::PacketParsingError("failed to parse int".to_owned()))?;
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
    async fn send(&mut self, command: LssCommand) -> DriverResult<()>;
    async fn receive(&mut self) -> DriverResult<LssResponse>;
}

const TIMEOUT: u64 = 10;

pub struct FramedSerialDriver {
    #[cfg(target_family = "windows")]
    framed_port: Mutex<tokio_util::codec::Framed<tokio_serial::Serial, LssCodec>>,
    #[cfg(not(target_family = "windows"))]
    framed_port: tokio_util::codec::Framed<tokio_serial::Serial, LssCodec>,
}

impl FramedSerialDriver {
    pub fn new(port: &str) -> DriverResult<FramedSerialDriver> {
        let settings = tokio_serial::SerialPortSettings {
            baud_rate: 115200,
            timeout: std::time::Duration::from_millis(TIMEOUT),
            ..Default::default()
        };
        let serial_port = tokio_serial::Serial::from_path(port, &settings)
            .map_err(|_| LssDriverError::FailedOpeningSerialPort)?;
        Ok(FramedSerialDriver {
            #[cfg(target_family = "windows")]
            framed_port: Mutex::new(LssCodec.framed(serial_port)),
            #[cfg(not(target_family = "windows"))]
            framed_port: LssCodec.framed(serial_port),
        })
    }

    pub fn with_baud_rate(port: &str, baud_rate: u32) -> DriverResult<FramedSerialDriver> {
        let settings = tokio_serial::SerialPortSettings {
            baud_rate,
            timeout: std::time::Duration::from_millis(TIMEOUT),
            ..Default::default()
        };
        let serial_port = tokio_serial::Serial::from_path(port, &settings)
            .map_err(|_| LssDriverError::FailedOpeningSerialPort)?;
        Ok(FramedSerialDriver {
            #[cfg(target_family = "windows")]
            framed_port: Mutex::new(LssCodec.framed(serial_port)),
            #[cfg(not(target_family = "windows"))]
            framed_port: LssCodec.framed(serial_port),
        })
    }
}

#[async_trait]
impl FramedDriver for FramedSerialDriver {
    async fn send(&mut self, command: LssCommand) -> DriverResult<()> {
        #[cfg(not(target_family = "windows"))]
        let port = &mut self.framed_port;
        #[cfg(target_family = "windows")]
        let mut port = self.framed_port.lock().await;
        port.send(command)
            .await
            .map_err(|_| LssDriverError::SendingError)?;
        Ok(())
    }

    async fn receive(&mut self) -> DriverResult<LssResponse> {
        #[cfg(not(target_family = "windows"))]
        let port = &mut self.framed_port;
        #[cfg(target_family = "windows")]
        let mut port = self.framed_port.lock().await;
        let response = timeout(Duration::from_millis(TIMEOUT), port.next())
            .await
            .map_err(|_| LssDriverError::TimeoutError)?
            .ok_or_else(|| {
                LssDriverError::PacketParsingError("Failed to extract message".to_owned())
            })?
            .map_err(|_| LssDriverError::PacketParsingError("Unknown error".to_owned()))?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn framing_returns_none() {
        let mut payload = BytesMut::from("*5QV11200");
        let mut codec = LssCodec {};
        let res = codec.decode(&mut payload).unwrap();
        assert_eq!(res, None);
    }

    #[test]
    fn framing_returns_twice() {
        let mut payload = BytesMut::from("*1QV1\r*2QV2\r");
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
        let mut payload = BytesMut::from("*5QV11200\r");
        let mut codec = LssCodec {};
        let res = codec.decode(&mut payload).unwrap().unwrap();
        let (id, val) = res.separate("QV").unwrap();
        assert_eq!(id, 5);
        assert_eq!(val, 11200);
    }

    #[test]
    fn framing_encodes_single_command() {
        let mut payload = BytesMut::default();
        let mut codec = LssCodec {};
        let command = LssCommand::simple(5, "QV");
        codec.encode(command, &mut payload).unwrap();
        assert_eq!(&payload[..], b"#5QV\r");
    }

    #[test]
    fn framing_encodes_multiple_commands() {
        let mut payload = BytesMut::default();
        let mut codec = LssCodec {};
        let command_1 = LssCommand::simple(5, "QV");
        let command_2 = LssCommand::simple(5, "QT");
        codec.encode(command_1, &mut payload).unwrap();
        codec.encode(command_2, &mut payload).unwrap();
        assert_eq!(&payload[..], b"#5QV\r#5QT\r");
    }

    #[test]
    fn simple_command_serializes() {
        let command = LssCommand::simple(1, "QV");
        assert_eq!(command.as_bytes(), b"#1QV\r")
    }

    #[test]
    fn param_command_serializes() {
        let command = LssCommand::with_param(1, "D", 10);
        assert_eq!(command.as_bytes(), b"#1D10\r")
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
