use bytes::{BytesMut, BufMut};
use std::{ str, io, error::Error };
use tokio_util::codec::{Decoder, Encoder};
use async_trait::async_trait;
use futures::{ SinkExt, StreamExt };


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

    fn as_bytes(&self) -> &[u8] {
        self.message.as_bytes()
    }
}

pub struct LssResponse {
    message: String
}

impl LssResponse {
    fn new(message: String) -> LssResponse {
        LssResponse {
            message
        }
    }

    pub fn separate(&self, separator: &str) -> Result<(u8, i32), Box<dyn Error>> {
        let mut split = self
            .message[1..]
            .trim()
            .split(separator);
        let id: u8 = split.next().ok_or("Failed to extract id")?.parse()?;
        let value: i32 = split.next().ok_or("Failed to extract value")?.parse()?;
        Ok((id, value))
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