/*!
 * # Iron LSS
 *
 * Iron LSS is a driver library for Lynxmotion Smart Servos
 *
 * You can read more about the LSS Servos here their
 * [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/)
 *
 */
use tokio_util::codec::{Decoder, Encoder};
use std::error::Error;
use std::str;
use std::io;
use futures::SinkExt;
use futures::stream::StreamExt;

use bytes::{BytesMut, BufMut};
// use futures_util::sink::SinkExt;


struct LssCodec;

impl Decoder for LssCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let command_break = src.as_ref().iter().position(|b| *b == b'\r');
        if let Some(n) = command_break {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LssCodec {
    type Error = io::Error;

    fn encode(&mut self, data: String, buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.reserve(data.len());
        buf.put(data.as_bytes());
        Ok(())
    }
}


#[derive(Copy, Clone)]
pub enum LedColor {
    Off = 0,
    Red = 1,
    Green = 2,
    Blue = 3,
    Yellow = 4,
    Cyan = 5,
    Magenta = 6,
    White = 7,
}

const TIMEOUT: u64 = 10;

/// Driver for the LSS servo
pub struct LSSDriver {
    port: tokio_util::codec::Framed<tokio_serial::Serial, LssCodec>,
}

impl LSSDriver {
    /// Create new driver on a serial port with default settings
    ///
    /// Default baud_rate is 115200
    ///
    /// # Arguments
    ///
    /// * `post` - Port to use. e.g. COM1 or /dev/ttyACM0
    ///
    /// # Example
    ///
    /// ```no_run
    /// use iron_lss::LSSDriver;
    /// let mut driver = LSSDriver::new("COM1").unwrap();
    /// ```
    pub fn new(port: &str) -> Result<LSSDriver, Box<dyn Error>> {
        let mut settings = tokio_serial::SerialPortSettings::default();
        settings.baud_rate = 115200;
        settings.timeout = std::time::Duration::from_millis(TIMEOUT);
        let serial_port = tokio_serial::Serial::from_path(port, &settings)?;
        Ok(LSSDriver {
            port: LssCodec.framed(serial_port),
        })
    }

    /// Create new driver on a serial port with custom baud rate
    ///
    /// # Arguments
    ///
    /// * `post` - Port to use. e.g. COM1 or /dev/ttyACM0
    /// * `baud_rate` - Baudrate. e.g. 115200
    ///
    /// # Example
    ///
    /// ```no_run
    /// use iron_lss::LSSDriver;
    /// let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    /// ```
    pub fn with_baud_rate(port: &str, baud_rate: u32) -> Result<LSSDriver, Box<dyn Error>> {
        let mut settings = tokio_serial::SerialPortSettings::default();
        settings.baud_rate = baud_rate;
        settings.timeout = std::time::Duration::from_millis(TIMEOUT);
        let serial_port = tokio_serial::Serial::from_path(port, &settings)?;
        Ok(LSSDriver {
            port: LssCodec.framed(serial_port),
        })
    }

    /// set color for driver with id
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `color` - Color to set
    pub async fn set_color(&mut self, id: u8, color: LedColor) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}LED{}\r", id, color as u8);
        self.port.send(message).await?;
        Ok(())
    }

    /// Move to absolute position in degrees
    ///
    /// Supports virtual positions that are more than 360 degrees
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `position` - Absolute position in degrees
    ///
    /// ```no_run
    /// use iron_lss::LSSDriver;
    /// let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    /// driver.move_to_position(5, 180.0).unwrap();
    /// driver.move_to_position(5, 480.0).unwrap();
    /// ```
    pub async fn move_to_position(&mut self, id: u8, position: f32) -> Result<(), Box<dyn Error>> {
        let angle = (position * 10.0).round() as i32;
        let message = format!("#{}D{}\r", id, angle);
        self.port.send(message).await?;
        Ok(())
    }

    /// Disables motion profile allowing servo to be directly controlled
    ///
    /// With motion profile enabled servos will follow a motion curve
    /// With motion profile disabled servos
    /// can be positionally controlled at high speed
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `motion_profile` - set motion profile on/off
    pub async fn disable_motion_profile(
        &mut self,
        id: u8,
        motion_profile: bool,
    ) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}EM{}\r", id, motion_profile as u8);
        self.port.send(message).await?;
        Ok(())
    }

    /// Disables power to motor allowing it to be back driven
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn limp(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}L\r", id);
        self.port.send(message).await?;
        Ok(())
    }

    /// Stops any ongoing motor motion and actively holds position
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn halt_hold(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}H\r", id);
        self.port.send(message).await?;
        Ok(())
    }

    /// Read current position of motor in degrees
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to read from
    pub async fn read_position(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        // response message looks like *5QDT6783<cr>
        // Response is in 10s of degrees
        let message = format!("#{}QDT\r", id);
        self.port.send(message).await?;
        let response = self.port.next().await.ok_or("failed reading from port")??;
        let text = response
            .split("QDT")
            .last()
            .ok_or("Response message is empty")?
            .trim();
        Ok(text.parse::<f32>()? / 10.0)
    }

    /// Read voltage of motor in volts
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to read from
    pub async fn read_voltage(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        // response message looks like *5QV11200<cr>
        // Response is in mV
        let message = format!("#{}QV\r", id);
        self.port.send(message).await?;
        let response = self.port.next().await.ok_or("failed reading from port")??;
        let text = response
            .split("QV")
            .last()
            .ok_or("Response message is empty")?
            .trim();
        Ok(text.parse::<f32>()? / 1000.0)
    }

    /// Read temperature of motor in celsius
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to read from
    pub async fn read_temperature(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        // response message looks like *5QT441<cr>
        // Response is in 10s of celsius
        // 441 would be 44.1 celsius
        let message = format!("#{}QT\r", id);
        self.port.send(message).await?;
        let response = self.port.next().await.ok_or("failed reading from port")??;
        let text = response
            .split("QT")
            .last()
            .ok_or("Response message is empty")?
            .trim();
        Ok(text.parse::<f32>()? / 10.0)
    }

    /// Read current of motor in Amps
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to read from
    pub async fn read_current(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        // response message looks like *5QT441<cr>
        // Response is in mA
        let message = format!("#{}QC\r", id);
        self.port.send(message).await?;
        if let Some(data) = self.port.next().await {
            let text = data?;
            let text = text
                .split("QC")
                .last()
                .ok_or("Response message is empty")?
                .trim();
            Ok(text.parse::<f32>()? / 1000.0)
        } else {
            Err("Failed reading")?
        }
    }
}
