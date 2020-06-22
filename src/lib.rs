use serialport;
use std::error::Error;

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

pub struct LSSDriver {
    port: Box<dyn serialport::SerialPort>,
}

impl LSSDriver {
    pub fn new(port: &str) -> Result<LSSDriver, Box<dyn Error>> {
        let mut settings = serialport::SerialPortSettings::default();
        settings.baud_rate = 115200;
        let serial_port = serialport::open_with_settings(port, &settings)?;
        Ok(LSSDriver {
            port: serial_port
        })
    }

    pub fn set_color(&mut self, id: u8, color: LedColor) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}LED{}\r", id, color as u8);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
        Ok(())
    }

    pub fn move_to_position(&mut self, id: u8, position: f32) -> Result<(), Box<dyn Error>> {
        let angle = (position * 10.0).round() as i32;
        let message = format!("#{}D{}\r", id, angle);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
        Ok(())
    }

    pub fn disable_motion_profile(&mut self, id: u8, motion_profile: bool) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}EM0\r", id);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
        Ok(())
    }
}

