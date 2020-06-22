/*!
 * # Iron LSS
 *
 * Iron LSS is a driver library for Lynxmotion Smart Servos
 *
 * You can read more about the LSS Servos here their
 * [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/)
 *
 */
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

/// Driver for the LSS servo
pub struct LSSDriver {
    port: Box<dyn serialport::SerialPort>,
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
    /// ```
    /// use iron_lss::LSSDriver;
    /// let mut driver = LSSDriver::new("COM1").unwrap();
    /// ```
    pub fn new(port: &str) -> Result<LSSDriver, Box<dyn Error>> {
        let mut settings = serialport::SerialPortSettings::default();
        settings.baud_rate = 115200;
        let serial_port = serialport::open_with_settings(port, &settings)?;
        Ok(LSSDriver { port: serial_port })
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
    /// ```
    /// use iron_lss::LSSDriver;
    /// let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    /// ```
    pub fn with_baud_rate(port: &str, baud_rate: u32) -> Result<LSSDriver, Box<dyn Error>> {
        let mut settings = serialport::SerialPortSettings::default();
        settings.baud_rate = baud_rate;
        let serial_port = serialport::open_with_settings(port, &settings)?;
        Ok(LSSDriver { port: serial_port })
    }

    /// set color for driver with id
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `color` - Color to set
    pub fn set_color(&mut self, id: u8, color: LedColor) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}LED{}\r", id, color as u8);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
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
    /// ```
    /// use iron_lss::LSSDriver;
    /// let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    /// driver.move_to_position(5, 180.0).unwrap();
    /// driver.move_to_position(5, 480.0).unwrap();
    /// ```
    pub fn move_to_position(&mut self, id: u8, position: f32) -> Result<(), Box<dyn Error>> {
        let angle = (position * 10.0).round() as i32;
        let message = format!("#{}D{}\r", id, angle);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
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
    pub fn disable_motion_profile(
        &mut self,
        id: u8,
        motion_profile: bool,
    ) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}EM{}\r", id, motion_profile as u8);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
        Ok(())
    }

    /// Disables power to motor allowing it to be back driven
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub fn limp(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}L\r", id);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
        Ok(())
    }

    /// Stops any ongoing motor motion and actively holds position
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub fn halt_hold(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}H\r", id);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
        Ok(())
    }
}
