/*!
 * # Iron LSS
 *
 * Iron LSS is a driver library for Lynxmotion Smart Servos
 *
 * You can read more about the LSS Servos here their
 * [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/)
 *
 */
mod serial_driver;

use serial_driver::{ FramedSerialDriver, FramedDriver, LssCommand };
use std::{ str, error::Error };


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
    driver: Box<dyn FramedDriver>,
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
        let driver = FramedSerialDriver::new(port)?;
        Ok(LSSDriver {
            driver: Box::new(driver),
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
        let driver = FramedSerialDriver::with_baud_rate(port, baud_rate)?;
        Ok(LSSDriver {
            driver: Box::new(driver),
        })
    }

    

    /// set color for driver with id
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `color` - Color to set
    pub async fn set_color(&mut self, id: u8, color: LedColor) -> Result<(), Box<dyn Error>> {
        self.driver.send(LssCommand::with_param(id, "LED", color as i32)).await?;
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
        self.driver.send(LssCommand::with_param(id, "D", angle)).await?;
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
        self.driver.send(LssCommand::with_param(id, "EM", motion_profile as i32)).await?;
        Ok(())
    }

    /// Set angular stiffness
    /// 
    /// Read more about [Angular stiffness](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularStiffness28AS29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `angular_stiffness` - value for angular stiffness (-10 to 10)
    pub async fn set_angular_stiffness(
        &mut self,
        id: u8,
        angular_stiffness: i32,
    ) -> Result<(), Box<dyn Error>> {
        self.driver.send(LssCommand::with_param(id, "AS", angular_stiffness)).await?;
        Ok(())
    }

    /// Set angular holding stiffness
    /// 
    /// Read more about [Angular holding stiffness](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularHoldingStiffness28AH29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `angular_holding` - value for angular holding stiffness (-10 to 10)
    pub async fn set_angular_holding(
        &mut self,
        id: u8,
        angular_holding: i32,
    ) -> Result<(), Box<dyn Error>> {
        self.driver.send(LssCommand::with_param(id, "AH", angular_holding)).await?;
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
        self.driver.send(LssCommand::with_param(id, "FPC", filter_position_count as i32)).await?;
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
        self.driver.send(LssCommand::simple(id, "QFPC")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QFPC")?;
        Ok(value as u8)
    }

    /// Disables power to motor allowing it to be back driven
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn limp(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "L")).await?;
        Ok(())
    }

    /// Stops any ongoing motor motion and actively holds position
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn halt_hold(&mut self, id: u8) -> Result<(), Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "H")).await?;
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
        self.driver.send(LssCommand::simple(id, "QDT")).await?;
        let response = self.driver.receive().await?;
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
        self.driver.send(LssCommand::simple(id, "QV")).await?;
        let response = self.driver.receive().await?;
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
        self.driver.send(LssCommand::simple(id, "QT")).await?;
        let response = self.driver.receive().await?;
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
        self.driver.send(LssCommand::simple(id, "QC")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QC")?;
        Ok(value as f32 / 1000.0)
    }
}
