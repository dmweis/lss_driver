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
use std::{ str, io, error::Error };
use futures::{ SinkExt, StreamExt };
use bytes::{BytesMut, BufMut};

struct LssCommand {
    message: String,
}

impl LssCommand {
    fn with_param(id: u8, cmd: &str, val: i32) -> LssCommand {
        LssCommand {
            message: format!("#{}{}{}\r", id, cmd, val),
        }
    }

    fn simple(id: u8, cmd: &str) -> LssCommand {
        LssCommand {
            message: format!("#{}{}\r", id, cmd),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        self.message.as_bytes()
    }
}

struct LssResponse {
    message: String
}

impl LssResponse {
    fn new(message: String) -> LssResponse {
        LssResponse {
            message
        }
    }

    fn separate(&self, separator: &str) -> Result<(u8, i32), Box<dyn Error>> {
        let mut split = self
            .message[1..]
            .trim()
            .split(separator);
        let id: u8 = split.next().ok_or("Failed to extract id")?.parse()?;
        let value: i32 = split.next().ok_or("Failed to extract value")?.parse()?;
        Ok((id, value))
    }
}

struct LssCodec;

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
    framed_port: tokio_util::codec::Framed<tokio_serial::Serial, LssCodec>,
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
            framed_port: LssCodec.framed(serial_port),
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
            framed_port: LssCodec.framed(serial_port),
        })
    }


    /// set color for driver with id
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `color` - Color to set
    pub async fn set_color(&mut self, id: u8, color: LedColor) -> Result<(), Box<dyn Error>> {
        self.framed_port.send(LssCommand::with_param(id, "LED", color as i32)).await?;
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
    /// async fn async_main(){
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.move_to_position(5, 180.0).await;
    ///     driver.move_to_position(5, 480.0).await;
    /// }
    /// ```
    pub async fn move_to_position(&mut self, id: u8, position: f32) -> Result<(), Box<dyn Error>> {
        let angle = (position * 10.0).round() as i32;
        self.framed_port.send(LssCommand::with_param(id, "D", angle)).await?;
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
    pub async fn set_motion_profile(
        &mut self,
        id: u8,
        motion_profile: bool,
    ) -> Result<(), Box<dyn Error>> {
        self.framed_port.send(LssCommand::with_param(id, "EM", motion_profile as i32)).await?;
        Ok(())
    }

    /// Set filter position count
    ///
    /// Change the Filter Position Count value for this session.
    /// Affects motion only when motion profile is disabled (EM0)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `filter_position_count` - default if 5
    pub async fn set_filter_position_count(
        &mut self,
        id: u8,
        filter_position_count: u8,
    ) -> Result<(), Box<dyn Error>> {
        self.framed_port.send(LssCommand::with_param(id, "FPC", filter_position_count as i32)).await?;
        Ok(())
    }

    /// Read filter position count
    ///
    /// Affects motion only when motion profile is disabled (EM0)
    /// Default is 5
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to read from
    pub async fn read_filter_position_count(
        &mut self,
        id: u8,
    ) -> Result<u8, Box<dyn Error>> {
        self.framed_port.send(LssCommand::simple(id, "QFPC")).await?;
        let response = self.framed_port.next().await.ok_or("Failed to parse response")??;
        let (_, value) = response.separate("QFPC")?;
        Ok(value as u8)
    }

    /// Disables power to motor allowing it to be back driven
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn limp(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        self.framed_port.send(LssCommand::simple(id, "L")).await?;
        Ok(())
    }

    /// Stops any ongoing motor motion and actively holds position
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn halt_hold(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        self.framed_port.send(LssCommand::simple(id, "H")).await?;
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
        self.framed_port.send(LssCommand::simple(id, "QDT")).await?;
        let response = self.framed_port.next().await.ok_or("Failed to parse response")??;
        let (_, value) = response.separate("QDT")?;
        Ok(value as f32 / 10.0)
    }

    /// Read voltage of motor in volts
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to read from
    pub async fn read_voltage(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        // response message looks like *5QV11200<cr>
        // Response is in mV
        self.framed_port.send(LssCommand::simple(id, "QV")).await?;
        let response = self.framed_port.next().await.ok_or("Failed to parse response")??;
        let (_, value) = response.separate("QV")?;
        Ok(value as f32 / 1000.0)
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
        self.framed_port.send(LssCommand::simple(id, "QT")).await?;
        let response = self.framed_port.next().await.ok_or("Failed to parse response")??;
        let (_, value) = response.separate("QT")?;
        Ok(value as f32 / 10.0)
    }

    /// Read current of motor in Amps
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to read from
    pub async fn read_current(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        // response message looks like *5QT441<cr>
        // Response is in mA
        self.framed_port.send(LssCommand::simple(id, "QC")).await?;
        let response = self.framed_port.next().await.ok_or("Failed to parse response")??;
        let (_, value) = response.separate("QC")?;
        Ok(value as f32 / 1000.0)
    }
}
