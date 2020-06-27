/*!
 * # LSS Driver
 *
 * lss_driver is a driver library for Lynxmotion Smart Servos
 *
 * You can read more about the LSS Servos here their
 * [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/)
 *
 */
mod serial_driver;

use serial_driver::{ FramedSerialDriver, FramedDriver, LssCommand };
use std::{ str, error::Error };


#[derive(Copy, Clone, Debug, PartialEq)]
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

impl LedColor {
    fn from_i32(number: i32) -> Result<LedColor, Box<dyn Error>> {
        match number {
            0 => Ok(LedColor::Off),
            1 => Ok(LedColor::Red),
            2 => Ok(LedColor::Green),
            3 => Ok(LedColor::Blue),
            4 => Ok(LedColor::Yellow),
            5 => Ok(LedColor::Cyan),
            6 => Ok(LedColor::Magenta),
            7 => Ok(LedColor::White),
            _ => Err("Could not find color")?,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MotorStatus {
    Unknown = 0,
    Limp = 1,
    FreeMoving = 2,
    Accelerating = 3,
    Traveling = 4,
    Decelerating = 5,
    Holding = 6,
    OutsideLimits = 7,
    Stuck = 8,
    Blocked = 9,
    SafeMode = 10,
}

impl MotorStatus {
    fn from_i32(number: i32) -> Result<MotorStatus, Box<dyn Error>> {
        match number {
            0 => Ok(MotorStatus::Unknown),
            1 => Ok(MotorStatus::Limp),
            2 => Ok(MotorStatus::FreeMoving),
            3 => Ok(MotorStatus::Accelerating),
            4 => Ok(MotorStatus::Traveling),
            5 => Ok(MotorStatus::Decelerating),
            6 => Ok(MotorStatus::Holding),
            7 => Ok(MotorStatus::OutsideLimits),
            8 => Ok(MotorStatus::Stuck),
            9 => Ok(MotorStatus::Blocked),
            10 => Ok(MotorStatus::SafeMode),
            _ => Err("Could not find status")?,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SafeModeStatus {
    NoLimits = 0,
    CurrentLimit = 1,
    InputVoltageOutOfRange = 2,
    TemperatureLimit = 3,
}

impl SafeModeStatus {
    fn from_i32(number: i32) -> Result<SafeModeStatus, Box<dyn Error>> {
        match number {
            0 => Ok(SafeModeStatus::NoLimits),
            1 => Ok(SafeModeStatus::CurrentLimit),
            2 => Ok(SafeModeStatus::InputVoltageOutOfRange),
            3 => Ok(SafeModeStatus::TemperatureLimit),
            _ => Err("Could not find safe status")?,
        }
    }
}

pub const BROADCAST_ID: u8 = 254;

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
    /// use lss_driver::LSSDriver;
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
    /// use lss_driver::LSSDriver;
    /// let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    /// ```
    pub fn with_baud_rate(port: &str, baud_rate: u32) -> Result<LSSDriver, Box<dyn Error>> {
        let driver = FramedSerialDriver::with_baud_rate(port, baud_rate)?;
        Ok(LSSDriver {
            driver: Box::new(driver),
        })
    }

    /// Creates new LSS driver with a custom implementation of the transport
    ///
    /// This is used for tests and can be used if you want to reimplement the driver over network
    pub fn with_driver(driver: Box<dyn FramedDriver>) -> LSSDriver {
        LSSDriver {
            driver,
        }
    }

    /// Query value of ID
    /// Especially useful with BROADCAST_ID
    /// 
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HIdentificationNumber28ID29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// 
    /// # Example
    ///
    /// ```no_run
    /// use lss_driver::LSSDriver;
    /// async fn async_main(){
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     let id = driver.query_id(lss_driver::BROADCAST_ID).await.unwrap();
    /// }
    /// ```
    pub async fn query_id(&mut self, id: u8) -> Result<u8, Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "QID")).await?;
        let response = self.driver.receive().await?;
        let value = response.get_val("QID")?;
        Ok(value as u8)
    }

    /// Set value of ID
    /// Saved to EEPROM
    /// Only takes effect after restart
    /// 
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HIdentificationNumber28ID29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `new_id` - ID You want that servo to have
    pub async fn set_id(&mut self, id: u8, new_id: u8) -> Result<(), Box<dyn Error>> {
        self.driver.send(LssCommand::with_param(id, "CID", new_id as i32)).await?;
        Ok(())
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

    /// Query color of servo LED
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_color(&mut self, id: u8) -> Result<LedColor, Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "QLED")).await?;
        let response = self.driver.receive().await?;
        let (_, value)= response.separate("QLED")?;
        Ok(LedColor::from_i32(value)?)
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
    /// # Example
    ///
    /// ```no_run
    /// use lss_driver::LSSDriver;
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

    /// Query absolute current position in degrees
    ///
    /// Supports virtual positions that are more than 360 degrees
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_position(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "QD")).await?;
        let response = self.driver.receive().await?;
        let (_, value)= response.separate("QD")?;
        Ok(value as f32 / 10.0)
    }

    /// Query absolute target position in degrees
    ///
    /// Supports virtual positions that are more than 360 degrees
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_target_position(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "QDT")).await?;
        let response = self.driver.receive().await?;
        let (_, value)= response.separate("QDT")?;
        Ok(value as f32 / 10.0)
    }

    /// Set continuous rotation speed in °/s
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `speed` - Speed in °/s
    pub async fn set_rotation_speed(&mut self, id: u8, speed: f32) -> Result<(), Box<dyn Error>> {
        self.driver.send(LssCommand::with_param(id, "WD", speed as i32)).await?;
        Ok(())
    }

    /// Query absolute rotation speed in °/s
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_rotation_speed(&mut self, id: u8) -> Result<f32, Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "QWD")).await?;
        let response = self.driver.receive().await?;
        let (_, value)= response.separate("QWD")?;
        Ok(value as f32)
    }

    /// Query status of a motor
    ///
    /// View more on [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HQueryStatus28Q29)
    /// 
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_status(&mut self, id: u8) -> Result<MotorStatus, Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "Q")).await?;
        let response = self.driver.receive().await?;
        let (_, value)= response.separate("Q")?;
        Ok(MotorStatus::from_i32(value)?)
    }

    /// Query safety status of a motor
    ///
    /// View more on [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HQueryStatus28Q29)
    /// 
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_safety_status(&mut self, id: u8) -> Result<SafeModeStatus, Box<dyn Error>> {
        self.driver.send(LssCommand::simple(id, "Q1")).await?;
        let response = self.driver.receive().await?;
        let (_, value)= response.separate("Q")?;
        Ok(SafeModeStatus::from_i32(value)?)
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


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use async_trait::async_trait;
    use super::serial_driver::LssResponse;


    struct MockedDriver {
        expected_send: Vec<String>,
        receive: Vec<String>,
    }

    #[async_trait]
    impl FramedDriver for MockedDriver {
        async fn send(&mut self, command: LssCommand) -> Result<(), Box<dyn Error>> {
            let expected = self.expected_send.pop().unwrap();
            assert_eq!(expected, command.as_str().to_owned());
            Ok(())
        }

        async fn receive(&mut self) -> Result<LssResponse, Box<dyn Error>> {
            Ok(LssResponse::new(self.receive.pop().unwrap()))
        }
    }

    #[tokio::test]
    async fn async_test_builds() {}


    #[tokio::test]
    async fn test_limp_color_move_hold() {
        let mocked_framed_driver = MockedDriver {
            expected_send: vec![
                "#5QV\r".to_owned(),
                "#4H\r".to_owned(),
                "#3D1800\r".to_owned(),
                "#2LED1\r".to_owned(),
                "#1L\r".to_owned(),
            ],
            receive: vec![
                "*5QV11200\r".to_owned(),
            ],
        };
        let mut driver = LSSDriver::with_driver(Box::new(mocked_framed_driver));
        driver.limp(1).await.unwrap();
        driver.set_color(2, LedColor::Red).await.unwrap();
        driver.move_to_position(3, 180.0).await.unwrap();
        driver.halt_hold(4).await.unwrap();
        let voltage = driver.read_voltage(5).await.unwrap();
        assert_eq!(voltage, 11.2);
    }

    macro_rules! test_command {
        ($name:ident, $expected:expr, $command:expr) => {
            #[tokio::test]
            async fn $name() {
                let mocked_framed_driver = MockedDriver {
                    expected_send: vec![
                        $expected.to_owned(),
                    ],
                    receive: vec![],
                };
                let mut driver = LSSDriver::with_driver(Box::new(mocked_framed_driver));
                $command;
            }
        }
    }

    macro_rules! test_query {
        ($name:ident, $expected:expr, $recv:expr, $command:expr, $val:expr) => {
            #[tokio::test]
            async fn $name() {
                let mocked_framed_driver = MockedDriver {
                    expected_send: vec![
                        $expected.to_owned(),
                    ],
                    receive: vec![
                        $recv.to_owned(),
                    ],
                };
                let mut driver = LSSDriver::with_driver(Box::new(mocked_framed_driver));
                let res = $command;
                assert_eq!(res, $val);
            }
        }
    }

    test_command!(test_hold_command, "#4H\r", driver.halt_hold(4).await.unwrap());
    test_query!(test_query_voltage, "#5QV\r", "*5QV11200\r", driver.read_voltage(5).await.unwrap(), 11.2);
    test_query!(test_query_id, "#254QID\r", "*QID5\r", driver.query_id(BROADCAST_ID).await.unwrap(), 5);
    test_command!(test_set_id, "#1CID2\r", driver.set_id(1, 2).await.unwrap());

    // Motion
    test_command!(test_move_to, "#1D200\r", driver.move_to_position(1, 20.0).await.unwrap());
    test_query!(test_query_current_position, "#5QD\r", "*5QD132\r", driver.query_position(5).await.unwrap(), 13.2);
    test_query!(test_query_target_position, "#5QDT\r", "*5QDT6783\r", driver.query_target_position(5).await.unwrap(), 678.3);
    
    // Wheel mode
    test_command!(test_set_rotation_speed_degrees, "#5WD90\r", driver.set_rotation_speed(5, 90.0).await.unwrap());
    test_query!(test_query_rotation_speed_degrees, "#5QWD\r", "*5QWD90\r", driver.query_rotation_speed(5).await.unwrap(), 90.0);
    
    // Status
    test_query!(test_unknown_status, "#5Q\r", "*5Q0\r", driver.query_status(5).await.unwrap(), MotorStatus::Unknown);
    test_query!(test_holding_status, "#5Q\r", "*5Q6\r", driver.query_status(5).await.unwrap(), MotorStatus::Holding);
    test_query!(test_safety_status, "#5Q1\r", "*5Q3\r", driver.query_safety_status(5).await.unwrap(), SafeModeStatus::TemperatureLimit);

    test_command!(test_limp, "#5L\r", driver.limp(5).await.unwrap());
    test_command!(test_halt_hold, "#5H\r", driver.halt_hold(5).await.unwrap());

    // LED
    test_command!(test_set_led, "#5LED3\r", driver.set_color(5, LedColor::Blue).await.unwrap());
    test_query!(test_query_led, "#5QLED\r", "*5QLED5\r", driver.query_color(5).await.unwrap(), LedColor::Cyan);

}
