#![doc = include_str!("../README.md")]

mod message_types;
mod serial_driver;

pub use message_types::*;
use serial_driver::{FramedDriver, FramedSerialDriver, LssCommand};
use std::str;

/// ID used to talk to all motors on a bus at once
pub const BROADCAST_ID: u8 = 254;

type DriverResult<T> = Result<T, LssDriverError>;

/// Driver for the LSS servo
pub struct LSSDriver {
    driver: Box<dyn FramedDriver + Send + Sync>,
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
    pub fn new(port: &str) -> DriverResult<LSSDriver> {
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
    pub fn with_baud_rate(port: &str, baud_rate: u32) -> DriverResult<LSSDriver> {
        let driver = FramedSerialDriver::with_baud_rate(port, baud_rate)?;
        Ok(LSSDriver {
            driver: Box::new(driver),
        })
    }

    /// Creates new LSS driver with a custom implementation of the transport
    ///
    /// This is used for tests and can be used if you want to reimplement the driver over network
    pub fn with_driver(driver: Box<dyn FramedDriver + Send + Sync>) -> LSSDriver {
        LSSDriver { driver }
    }

    /// Soft reset
    /// This command does a "soft reset" and reverts all commands to those stored in EEPROM
    ///
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HReset)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to reset
    pub async fn reset(&mut self, id: u8) -> DriverResult<()> {
        self.driver.send(LssCommand::simple(id, "RESET")).await?;
        Ok(())
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
    pub async fn query_id(&mut self, id: u8) -> DriverResult<u8> {
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
    pub async fn set_id(&mut self, id: u8, new_id: u8) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "CID", new_id as i32))
            .await?;
        Ok(())
    }

    /// set color for driver with id
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `color` - Color to set
    pub async fn set_color(&mut self, id: u8, color: LedColor) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "LED", color as i32))
            .await?;
        Ok(())
    }

    /// Query color of servo LED
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_color(&mut self, id: u8) -> DriverResult<LedColor> {
        self.driver.send(LssCommand::simple(id, "QLED")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QLED")?;
        LedColor::from_i32(value)
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
    pub async fn move_to_position(&mut self, id: u8, position: f32) -> DriverResult<()> {
        let angle = (position * 10.0).round() as i32;
        self.driver
            .send(LssCommand::with_param(id, "D", angle))
            .await?;
        Ok(())
    }

    /// Move to absolute position in degrees with modifier
    ///
    /// Supports virtual positions that are more than 360 degrees
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `position` - Absolute position in degrees
    /// * `modifier` - Modifier applied to this motion. Look at the type for more info.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lss_driver::LSSDriver;
    /// use lss_driver::CommandModifier;
    /// async fn async_main(){
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.move_to_position_with_modifier(5, 180.0, CommandModifier::Timed(2500)).await;
    ///     driver.move_to_position_with_modifier(5, 480.0, CommandModifier::Timed(2500)).await;
    /// }
    /// ```
    pub async fn move_to_position_with_modifier(
        &mut self,
        id: u8,
        position: f32,
        modifier: CommandModifier,
    ) -> DriverResult<()> {
        let angle = (position * 10.0).round() as i32;
        self.driver
            .send(LssCommand::with_param_modifier(id, "D", angle, modifier))
            .await?;
        Ok(())
    }

    /// Move to absolute position in degrees with multiple modifiers
    ///
    /// Supports virtual positions that are more than 360 degrees
    /// Be careful about which modifiers are supported together
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `position` - Absolute position in degrees
    /// * `modifiers` - Array of modifiers applied to this motion. Look at the type for more info.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lss_driver::LSSDriver;
    /// use lss_driver::CommandModifier;
    /// async fn async_main(){
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.move_to_position_with_modifiers(5, 180.0, &[CommandModifier::Timed(2500), CommandModifier::CurrentHold(400)]).await;
    /// }
    /// ```
    pub async fn move_to_position_with_modifiers(
        &mut self,
        id: u8,
        position: f32,
        modifiers: &[CommandModifier],
    ) -> DriverResult<()> {
        let angle = (position * 10.0).round() as i32;
        self.driver
            .send(LssCommand::with_param_modifiers(id, "D", angle, modifiers))
            .await?;
        Ok(())
    }

    /// Move to absolute position in degrees
    ///
    /// Same as `move_to_position`
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
    ///     driver.set_target_position(5, 180.0).await;
    ///     driver.set_target_position(5, 480.0).await;
    /// }
    /// ```
    pub async fn set_target_position(&mut self, id: u8, position: f32) -> DriverResult<()> {
        self.move_to_position(id, position).await
    }

    /// Query absolute current position in degrees
    ///
    /// Supports virtual positions that are more than 360 degrees
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_position(&mut self, id: u8) -> DriverResult<f32> {
        self.driver.send(LssCommand::simple(id, "QD")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QD")?;
        Ok(value as f32 / 10.0)
    }

    /// Query absolute target position in degrees
    ///
    /// Supports virtual positions that are more than 360 degrees
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_target_position(&mut self, id: u8) -> DriverResult<f32> {
        self.driver.send(LssCommand::simple(id, "QDT")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QDT")?;
        Ok(value as f32 / 10.0)
    }

    /// Set continuous rotation speed in °/s
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `speed` - Speed in °/s
    pub async fn set_rotation_speed(&mut self, id: u8, speed: f32) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "WD", speed as i32))
            .await?;
        Ok(())
    }

    /// Set continuous rotation speed in °/s with modifier
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `speed` - Speed in °/s
    /// * `modifier` - Modifier applied to this motion. Look at the type for more info.
    pub async fn set_rotation_speed_with_modifier(
        &mut self,
        id: u8,
        speed: f32,
        modifier: CommandModifier,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param_modifier(
                id,
                "WD",
                speed as i32,
                modifier,
            ))
            .await?;
        Ok(())
    }

    /// Query absolute rotation speed in °/s
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_rotation_speed(&mut self, id: u8) -> DriverResult<f32> {
        self.driver.send(LssCommand::simple(id, "QWD")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QWD")?;
        Ok(value as f32)
    }

    /// Query status of a motor
    ///
    /// View more on [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HQueryStatus28Q29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_status(&mut self, id: u8) -> DriverResult<MotorStatus> {
        self.driver.send(LssCommand::simple(id, "Q")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("Q")?;
        MotorStatus::from_i32(value)
    }

    /// Query safety status of a motor
    ///
    /// View more on [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HQueryStatus28Q29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_safety_status(&mut self, id: u8) -> DriverResult<SafeModeStatus> {
        self.driver.send(LssCommand::simple(id, "Q1")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("Q")?;
        SafeModeStatus::from_i32(value)
    }

    /// Set motion profile enabled or disabled.
    /// If the motion profile is enabled, angular acceleration (AA) and angular deceleration(AD) will have an effect on the motion. Also, SD/S and T modifiers can be used.
    ///
    /// With motion profile enabled servos will follow a motion curve
    /// With motion profile disabled servos move towards target location at full speed
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `motion_profile` - set motion profile on/off
    pub async fn set_motion_profile(&mut self, id: u8, motion_profile: bool) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "EM", motion_profile as i32))
            .await?;
        Ok(())
    }

    /// query motion profile enabled or disabled.
    /// If the motion profile is enabled, angular acceleration (AA) and angular deceleration(AD) will have an effect on the motion. Also, SD/S and T modifiers can be used.
    ///
    /// With motion profile enabled servos will follow a motion curve
    /// With motion profile disabled servos move towards target location at full speed
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_motion_profile(&mut self, id: u8) -> DriverResult<bool> {
        self.driver.send(LssCommand::simple(id, "QEM")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QEM")?;
        Ok(value != 0)
    }

    /// Set filter position count
    ///
    /// Change the Filter Position Count value for this session.
    /// Affects motion only when motion profile is disabled (EM0)
    ///
    /// more info at the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HFilterPositionCount28FPC29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `filter_position_count` - default if 5
    pub async fn set_filter_position_count(
        &mut self,
        id: u8,
        filter_position_count: u8,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(
                id,
                "FPC",
                filter_position_count as i32,
            ))
            .await?;
        Ok(())
    }

    /// Query filter position count
    ///
    /// Query the Filter Position Count value.
    /// Affects motion only when motion profile is disabled (EM0)
    ///
    /// more info at the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HFilterPositionCount28FPC29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_filter_position_count(&mut self, id: u8) -> DriverResult<u8> {
        self.driver.send(LssCommand::simple(id, "QFPC")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QFPC")?;
        Ok(value as u8)
    }

    /// Set angular stiffness
    ///
    /// Read more about [Angular stiffness](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularStiffness28AS29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `angular_stiffness` - value for angular stiffness (-10 to 10) (recommended -4 to 4)
    pub async fn set_angular_stiffness(
        &mut self,
        id: u8,
        angular_stiffness: i32,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "AS", angular_stiffness))
            .await?;
        Ok(())
    }

    /// Query angular stiffness
    ///
    /// Read more about [Angular stiffness](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularStiffness28AS29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_angular_stiffness(&mut self, id: u8) -> DriverResult<i32> {
        self.driver.send(LssCommand::simple(id, "QAS")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QAS")?;
        Ok(value)
    }

    /// Set angular holding stiffness
    ///
    /// Read more about [Angular holding stiffness](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularHoldingStiffness28AH29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `angular_holding` - value for angular holding stiffness (-10 to 10)
    pub async fn set_angular_holding_stiffness(
        &mut self,
        id: u8,
        angular_holding: i32,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "AH", angular_holding))
            .await?;
        Ok(())
    }

    /// Query angular holding stiffness
    ///
    /// Read more about [Angular holding stiffness](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularHoldingStiffness28AH29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn query_angular_holding_stiffness(&mut self, id: u8) -> DriverResult<i32> {
        self.driver.send(LssCommand::simple(id, "QAH")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QAH")?;
        Ok(value)
    }

    /// Set angular acceleration in degrees per second squared (°/s2)
    ///
    /// Accepts values between 1 and 100. Increments of 10
    /// Only used when motion profile is enabled
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularAcceleration28AA29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `angular_acceleration` - value for angular acceleration (1 to 100, Increments 10)
    pub async fn set_angular_acceleration(
        &mut self,
        id: u8,
        angular_acceleration: i32,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "AA", angular_acceleration))
            .await?;
        Ok(())
    }

    /// Query angular acceleration in degrees per second squared (°/s2)
    ///
    /// Accepts values between 1 and 100. Increments  of 10
    /// Only used when motion profile is enabled
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularAcceleration28AA29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_angular_acceleration(&mut self, id: u8) -> DriverResult<i32> {
        self.driver.send(LssCommand::simple(id, "QAA")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QAA")?;
        Ok(value)
    }

    /// Set angular deceleration in degrees per second squared (°/s2)
    ///
    /// Accepts values between 1 and 100. Increments of 10
    /// Only used when motion profile is enabled
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularDeceleration28AD29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `angular_deceleration` - value for angular deceleration (1 to 100, Increments 10)
    pub async fn set_angular_deceleration(
        &mut self,
        id: u8,
        angular_deceleration: i32,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "AD", angular_deceleration))
            .await?;
        Ok(())
    }

    /// Query angular deceleration in degrees per second squared (°/s2)
    ///
    /// Accepts values between 1 and 100. Increments  of 10
    /// Only used when motion profile is enabled
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularDeceleration28AD29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_angular_deceleration(&mut self, id: u8) -> DriverResult<i32> {
        self.driver.send(LssCommand::simple(id, "QAD")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QAD")?;
        Ok(value)
    }

    /// Set maximum motor duty
    ///
    /// Accepts values between 255 and 1023
    /// Only used when motion profile is disabled
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HMaximumMotorDuty28MMD29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `maximum_motor_duty` - value for maximum motor duty (255 to 1023)
    pub async fn set_maximum_motor_duty(
        &mut self,
        id: u8,
        maximum_motor_duty: i32,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "MMD", maximum_motor_duty))
            .await?;
        Ok(())
    }

    /// Query maximum motor duty
    ///
    /// Accepts values between 255 and 1023
    /// Only used when motion profile is disabled
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HMaximumMotorDuty28MMD29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_maximum_motor_duty(&mut self, id: u8) -> DriverResult<i32> {
        self.driver.send(LssCommand::simple(id, "QMMD")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QMMD")?;
        Ok(value)
    }

    /// Set maximum speed in degrees per second
    ///
    /// Accepts values up to 180.0
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HMaximumSpeedinDegrees28SD29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `maximum_speed` - value for maximum speed
    pub async fn set_maximum_speed(&mut self, id: u8, maximum_speed: f32) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(
                id,
                "SD",
                (maximum_speed * 10.) as i32,
            ))
            .await?;
        Ok(())
    }

    /// Query maximum speed in degrees per second
    ///
    /// Accepts values up to 180.0
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HMaximumSpeedinDegrees28SD29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_maximum_speed(&mut self, id: u8) -> DriverResult<f32> {
        self.driver.send(LssCommand::simple(id, "QSD")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QSD")?;
        Ok(value as f32 / 10.)
    }

    /// Disables power to motor allowing it to be back driven
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn limp(&mut self, id: u8) -> DriverResult<()> {
        self.driver.send(LssCommand::simple(id, "L")).await?;
        Ok(())
    }

    /// Stops any ongoing motor motion and actively holds position
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    pub async fn halt_hold(&mut self, id: u8) -> DriverResult<()> {
        self.driver.send(LssCommand::simple(id, "H")).await?;
        Ok(())
    }

    /// Query voltage of motor in volts
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to Query
    pub async fn query_voltage(&mut self, id: u8) -> DriverResult<f32> {
        // response message looks like *5QV11200<cr>
        // Response is in mV
        self.driver.send(LssCommand::simple(id, "QV")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QV")?;
        Ok(value as f32 / 1000.0)
    }

    /// Query temperature of motor in celsius
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to Query
    pub async fn query_temperature(&mut self, id: u8) -> DriverResult<f32> {
        // response message looks like *5QT441<cr>
        // Response is in 10s of celsius
        // 441 would be 44.1 celsius
        self.driver.send(LssCommand::simple(id, "QT")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QT")?;
        Ok(value as f32 / 10.0)
    }

    /// Query current of motor in Amps
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to Query
    pub async fn query_current(&mut self, id: u8) -> DriverResult<f32> {
        // response message looks like *5QT441<cr>
        // Response is in mA
        self.driver.send(LssCommand::simple(id, "QC")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QC")?;
        Ok(value as f32 / 1000.0)
    }

    /// Query model string
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_model(&mut self, id: u8) -> DriverResult<Model> {
        self.driver.send(LssCommand::simple(id, "QMS")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate_string("QMS")?;
        Ok(Model::from_str(&value))
    }

    /// Query firmware version
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_firmware_version(&mut self, id: u8) -> DriverResult<String> {
        self.driver.send(LssCommand::simple(id, "QF")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate_string("QF")?;
        Ok(value)
    }

    /// Query serial number
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    pub async fn query_serial_number(&mut self, id: u8) -> DriverResult<String> {
        self.driver.send(LssCommand::simple(id, "QN")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate_string("QN")?;
        Ok(value)
    }

    /// Set LED blinking mode
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HConfigureLEDBlinking28CLB29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `blinking_mode` - Blinking mode desired. Can be combination to make motor blink during multiple modes
    pub async fn set_led_blinking(
        &mut self,
        id: u8,
        blinking_mode: Vec<LedBlinking>,
    ) -> DriverResult<()> {
        let sum = blinking_mode
            .iter()
            .map(|item| *item as i32)
            .sum::<i32>()
            .min(LedBlinking::AlwaysBlink as i32);
        self.driver
            .send(LssCommand::with_param(id, "CLB", sum))
            .await?;
        Ok(())
    }

    /// Query origin offset in degrees
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HOriginOffset28O29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to query
    ///
    /// # Example
    /// ```no_run
    /// use lss_driver::LSSDriver;
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     let origin_offset = driver.query_origin_offset(5).await;
    /// }
    /// ```
    pub async fn query_origin_offset(&mut self, id: u8) -> DriverResult<f32> {
        // response messages looks like *5QO-13
        // Response is in tenths of degrees
        self.driver.send(LssCommand::simple(id, "QO")).await?;
        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QO")?;
        Ok(value as f32 / 10.0)
    }

    /// Query the angular range in degrees
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularRange28AR29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the servo you want to query
    ///
    ///  /// # Example
    /// ```no_run
    /// use lss_driver::LSSDriver;
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     let angular_range = driver.query_angular_range(5).await.unwrap();
    /// }
    /// ```
    pub async fn query_angular_range(&mut self, id: u8) -> DriverResult<f32> {
        // Response looks like *5QAR1800 where 1800 is range in 1/10 degrees
        // Contrary what the wiki says, which is *5AR1800 as example, the servo I used (HT1) returns
        // *5QAR1800
        self.driver.send(LssCommand::simple(id, "QAR")).await?;

        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QAR")?;

        Ok(value as f32 / 10.0)
    }

    /// Set the angular range in degrees
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HAngularRange28AR29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the servo you want to control
    /// * `range` - Angular range in degrees
    ///
    /// # Example
    /// ```no_run
    /// use lss_driver::LSSDriver;
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.set_angular_range(5, 180.0).await;
    /// }
    pub async fn set_angular_range(&mut self, id: u8, range: f32) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "CAR", (range * 10.) as i32))
            .await?;

        Ok(())
    }

    /// Queries the position in µs.
    ///
    /// When the position is inside the angular range it will return a value between 500 and 2500,
    /// when the position is outside the angular range it will return either -500 or -2500.
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HPositioninPWM28P29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the servo you want to control
    ///
    /// # Example
    /// ```no_run
    /// use lss_driver::LSSDriver;
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     let pwm_position = driver.query_pwm_position(5).await.unwrap();
    /// }
    /// ```
    pub async fn query_pwm_position(&mut self, id: u8) -> DriverResult<i32> {
        // Response looks like *5QP2334 where 2335 is in µs
        self.driver.send(LssCommand::simple(id, "QP")).await?;

        let response = self.driver.receive().await?;
        let (_, value) = response.separate("QP")?;

        Ok(value)
    }

    /// Set origin offset in degrees
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HOriginOffset28O29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of servo you want to control
    /// * `origin_offset` - Offset from factory 0 in degrees
    ///
    /// # Example
    /// ```no_run
    /// use lss_driver::LSSDriver;
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.set_origin_offset(5, -1.3).await;
    /// }
    /// ```
    pub async fn set_origin_offset(&mut self, id: u8, origin_offset: f32) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(
                id,
                "CO",
                (origin_offset * 10.) as i32,
            ))
            .await?;
        Ok(())
    }

    /// Move to PWM position in µs.
    ///
    /// You can use [set_angular_range](LSSDriver::set_angular_range) to range.
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HPositioninPWM28P29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the servo you want to control
    /// * `position` - Position in µs in the range [500,2500]
    ///
    /// # Example
    /// ```no_run
    /// use lss_driver::LSSDriver;
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.move_to_pwm_position(5, 2334).await;
    /// }
    /// ```
    pub async fn move_to_pwm_position(&mut self, id: u8, position: i32) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param(id, "P", position))
            .await?;

        Ok(())
    }

    /// Move to PWM position in µs with modifier.
    ///
    /// You can use [set_angular_range](LSSDriver::set_angular_range) to range.
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HPositioninPWM28P29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the servo you want to control
    /// * `position` - Position in µs in the range [500,2500]
    /// * `modifier` - Modifier applied to this motion. Look at the type for more info.
    ///
    /// # Example
    /// ```no_run
    /// use lss_driver::{LSSDriver, CommandModifier};
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.move_to_pwm_position_with_modifier(5, 2334, CommandModifier::Speed(750)).await;
    /// }
    /// ```
    pub async fn move_to_pwm_position_with_modifier(
        &mut self,
        id: u8,
        position: i32,
        modifier: CommandModifier,
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param_modifier(id, "P", position, modifier))
            .await?;

        Ok(())
    }

    /// Move to PWM position in µs with modifiers.
    ///
    /// You can use [set_angular_range](LSSDriver::set_angular_range) to range.
    ///
    /// Read more on the [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HPositioninPWM28P29)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the servo you want to control
    /// * `position` - Position in µs in the range [500,2500]
    /// * `modifiers` - Array of modifiers applied to this motion. Look at the type for more info.
    ///
    /// # Example
    /// ```no_run
    /// use lss_driver::{LSSDriver, CommandModifier};
    ///
    /// async fn async_main() {
    ///     let mut driver = LSSDriver::with_baud_rate("COM1", 115200).unwrap();
    ///     driver.move_to_pwm_position_with_modifiers(5, 2334, &[CommandModifier::Speed(750), CommandModifier::Timed(2500)]).await;
    /// }
    /// ```
    pub async fn move_to_pwm_position_with_modifiers(
        &mut self,
        id: u8,
        position: i32,
        modifiers: &[CommandModifier],
    ) -> DriverResult<()> {
        self.driver
            .send(LssCommand::with_param_modifiers(
                id, "P", position, modifiers,
            ))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::serial_driver::LssResponse;
    use super::*;
    use approx::assert_relative_eq;
    use async_trait::async_trait;

    struct MockedDriver {
        expected_send: Vec<String>,
        receive: Vec<String>,
    }

    #[async_trait]
    impl FramedDriver for MockedDriver {
        async fn send(&mut self, command: LssCommand) -> DriverResult<()> {
            let expected = self.expected_send.pop().unwrap();
            assert_eq!(expected, command.as_str().to_owned());
            Ok(())
        }

        async fn receive(&mut self) -> DriverResult<LssResponse> {
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
            receive: vec!["*5QV11200\r".to_owned()],
        };
        let mut driver = LSSDriver::with_driver(Box::new(mocked_framed_driver));
        driver.limp(1).await.unwrap();
        driver.set_color(2, LedColor::Red).await.unwrap();
        driver.move_to_position(3, 180.0).await.unwrap();
        driver.halt_hold(4).await.unwrap();
        let voltage = driver.query_voltage(5).await.unwrap();
        assert_relative_eq!(voltage, 11.2);
    }

    macro_rules! test_command {
        ($name:ident, $expected:expr, $command:expr) => {
            #[tokio::test]
            #[allow(clippy::redundant_closure_call)]
            async fn $name() {
                let mocked_framed_driver = MockedDriver {
                    expected_send: vec![$expected.to_owned()],
                    receive: vec![],
                };
                let driver = LSSDriver::with_driver(Box::new(mocked_framed_driver));
                ($command)(driver).await;
            }
        };
    }

    macro_rules! test_query {
        ($name:ident, $expected:expr, $recv:expr, $command:expr, $val:expr) => {
            #[tokio::test]
            #[allow(clippy::redundant_closure_call)]
            async fn $name() {
                let mocked_framed_driver = MockedDriver {
                    expected_send: vec![$expected.to_owned()],
                    receive: vec![$recv.to_owned()],
                };
                let driver = LSSDriver::with_driver(Box::new(mocked_framed_driver));
                let res = ($command)(driver).await;
                assert_eq!(res, $val);
            }
        };
    }

    macro_rules! test_query_float {
        ($name:ident, $expected:expr, $recv:expr, $command:expr, $val:expr) => {
            #[tokio::test]
            #[allow(clippy::redundant_closure_call)]
            async fn $name() {
                let mocked_framed_driver = MockedDriver {
                    expected_send: vec![$expected.to_owned()],
                    receive: vec![$recv.to_owned()],
                };
                let driver = LSSDriver::with_driver(Box::new(mocked_framed_driver));
                let res = ($command)(driver).await;
                assert_relative_eq!(res, $val);
            }
        };
    }

    test_command!(
        test_hold_command,
        "#4H\r",
        |mut driver: LSSDriver| async move {
            driver.halt_hold(4_u8).await.unwrap();
        }
    );

    test_query!(
        test_query_id,
        "#254QID\r",
        "*QID5\r",
        |mut driver: LSSDriver| async move { driver.query_id(BROADCAST_ID).await.unwrap() },
        5
    );
    test_command!(
        test_set_id,
        "#1CID2\r",
        |mut driver: LSSDriver| async move { driver.set_id(1, 2).await.unwrap() }
    );

    // Motion
    test_command!(
        test_move_to,
        "#1D200\r",
        |mut driver: LSSDriver| async move { driver.move_to_position(1, 20.0).await.unwrap() }
    );
    test_command!(
        test_move_to_with_modifier,
        "#1D200T15000\r",
        |mut driver: LSSDriver| async move {
            driver
                .move_to_position_with_modifier(1, 20.0, CommandModifier::Timed(15000))
                .await
                .unwrap()
        }
    );
    test_command!(
        test_move_to_with_modifiers,
        "#1D200T15000CL400\r",
        |mut driver: LSSDriver| async move {
            driver
                .move_to_position_with_modifiers(
                    1,
                    20.0,
                    &[
                        CommandModifier::Timed(15000),
                        CommandModifier::CurrentLimp(400),
                    ],
                )
                .await
                .unwrap()
        }
    );
    test_command!(
        test_set_target_position,
        "#1D200\r",
        |mut driver: LSSDriver| async move { driver.set_target_position(1, 20.0).await.unwrap() }
    );
    test_query_float!(
        test_query_current_position,
        "#5QD\r",
        "*5QD132\r",
        |mut driver: LSSDriver| async move { driver.query_position(5).await.unwrap() },
        13.2
    );
    test_query_float!(
        test_query_target_position,
        "#5QDT\r",
        "*5QDT6783\r",
        |mut driver: LSSDriver| async move { driver.query_target_position(5).await.unwrap() },
        678.3
    );

    // Wheel mode
    test_command!(
        test_set_rotation_speed_degrees,
        "#5WD90\r",
        |mut driver: LSSDriver| async move { driver.set_rotation_speed(5, 90.0).await.unwrap() }
    );
    test_command!(
        test_set_rotation_speed_degrees_with_modifier,
        "#1WD90T15000\r",
        |mut driver: LSSDriver| async move {
            driver
                .set_rotation_speed_with_modifier(1, 90.0, CommandModifier::Timed(15000))
                .await
                .unwrap()
        }
    );
    test_query_float!(
        test_query_rotation_speed_degrees,
        "#5QWD\r",
        "*5QWD90\r",
        |mut driver: LSSDriver| async move { driver.query_rotation_speed(5).await.unwrap() },
        90.0
    );

    // Status
    test_query!(
        test_unknown_status,
        "#5Q\r",
        "*5Q0\r",
        |mut driver: LSSDriver| async move { driver.query_status(5).await.unwrap() },
        MotorStatus::Unknown
    );
    test_query!(
        test_holding_status,
        "#5Q\r",
        "*5Q6\r",
        |mut driver: LSSDriver| async move { driver.query_status(5).await.unwrap() },
        MotorStatus::Holding
    );
    test_query!(
        test_safety_status,
        "#5Q1\r",
        "*5Q3\r",
        |mut driver: LSSDriver| async move { driver.query_safety_status(5).await.unwrap() },
        SafeModeStatus::TemperatureLimit
    );

    test_command!(test_limp, "#5L\r", |mut driver: LSSDriver| async move {
        driver.limp(5).await.unwrap()
    });
    test_command!(
        test_halt_hold,
        "#5H\r",
        |mut driver: LSSDriver| async move { driver.halt_hold(5).await.unwrap() }
    );

    // LED
    test_command!(
        test_set_led,
        "#5LED3\r",
        |mut driver: LSSDriver| async move { driver.set_color(5, LedColor::Blue).await.unwrap() }
    );
    test_query!(
        test_query_led,
        "#5QLED\r",
        "*5QLED5\r",
        |mut driver: LSSDriver| async move { driver.query_color(5).await.unwrap() },
        LedColor::Cyan
    );

    // motion profile
    test_command!(
        test_motion_profile_on,
        "#5EM1\r",
        |mut driver: LSSDriver| async move { driver.set_motion_profile(5, true).await.unwrap() }
    );
    test_command!(
        test_motion_profile_off,
        "#5EM0\r",
        |mut driver: LSSDriver| async move { driver.set_motion_profile(5, false).await.unwrap() }
    );
    test_query!(
        test_query_motion_profile_on,
        "#5QEM\r",
        "*5QEM1\r",
        |mut driver: LSSDriver| async move { driver.query_motion_profile(5).await.unwrap() },
        true
    );
    test_query!(
        test_query_motion_profile_off,
        "#5QEM\r",
        "*5QEM0\r",
        |mut driver: LSSDriver| async move { driver.query_motion_profile(5).await.unwrap() },
        false
    );

    test_command!(
        test_set_filter_position_count,
        "#5FPC10\r",
        |mut driver: LSSDriver| async move { driver.set_filter_position_count(5, 10).await.unwrap() }
    );
    test_query!(
        test_query_filter_position_count,
        "#5QFPC\r",
        "*5QFPC10\r",
        |mut driver: LSSDriver| async move { driver.query_filter_position_count(5).await.unwrap() },
        10
    );

    test_command!(
        test_set_angular_stiffness,
        "#5AS-2\r",
        |mut driver: LSSDriver| async move { driver.set_angular_stiffness(5, -2).await.unwrap() }
    );
    test_query!(
        test_query_angular_stiffness,
        "#5QAS\r",
        "*5QAS-2\r",
        |mut driver: LSSDriver| async move { driver.query_angular_stiffness(5).await.unwrap() },
        -2
    );

    test_command!(
        test_set_angular_holding_stiffness,
        "#5AH3\r",
        |mut driver: LSSDriver| async move { driver.set_angular_holding_stiffness(5, 3).await.unwrap() }
    );
    test_query!(
        test_query_angular_holding_stiffness,
        "#5QAH\r",
        "*5QAH3\r",
        |mut driver: LSSDriver| async move { driver.query_angular_holding_stiffness(5).await.unwrap() },
        3
    );

    test_command!(
        test_set_angular_acceleration,
        "#5AA30\r",
        |mut driver: LSSDriver| async move { driver.set_angular_acceleration(5, 30).await.unwrap() }
    );
    test_query!(
        test_query_angular_acceleration,
        "#5QAA\r",
        "*5QAA30\r",
        |mut driver: LSSDriver| async move { driver.query_angular_acceleration(5).await.unwrap() },
        30
    );

    test_command!(
        test_set_angular_deceleration,
        "#5AD30\r",
        |mut driver: LSSDriver| async move { driver.set_angular_deceleration(5, 30).await.unwrap() }
    );
    test_query!(
        test_query_angular_deceleration,
        "#5QAD\r",
        "*5QAD30\r",
        |mut driver: LSSDriver| async move { driver.query_angular_deceleration(5).await.unwrap() },
        30
    );

    test_command!(
        test_maximum_motor_duty,
        "#5MMD512\r",
        |mut driver: LSSDriver| async move { driver.set_maximum_motor_duty(5, 512).await.unwrap() }
    );
    test_query!(
        test_query_maximum_motor_duty,
        "#5QMMD\r",
        "*5QMMD512\r",
        |mut driver: LSSDriver| async move { driver.query_maximum_motor_duty(5).await.unwrap() },
        512
    );

    test_command!(
        test_maximum_speed,
        "#5SD1800\r",
        |mut driver: LSSDriver| async move { driver.set_maximum_speed(5, 180.).await.unwrap() }
    );
    test_query_float!(
        test_query_maximum_speed,
        "#5QSD\r",
        "*5QSD1800\r",
        |mut driver: LSSDriver| async move { driver.query_maximum_speed(5).await.unwrap() },
        180.
    );

    // test telemetry queries
    test_query_float!(
        test_query_voltage,
        "#5QV\r",
        "*5QV11200\r",
        |mut driver: LSSDriver| async move { driver.query_voltage(5).await.unwrap() },
        11.2
    );
    test_query_float!(
        test_query_temperature,
        "#5QT\r",
        "*5QT564\r",
        |mut driver: LSSDriver| async move { driver.query_temperature(5).await.unwrap() },
        56.4
    );
    test_query_float!(
        test_query_current,
        "#5QC\r",
        "*5QC140\r",
        |mut driver: LSSDriver| async move { driver.query_current(5).await.unwrap() },
        0.14
    );

    test_query!(
        test_query_model_string,
        "#5QMS\r",
        "*5QMSLSS-HS1\r",
        |mut driver: LSSDriver| async move { driver.query_model(5).await.unwrap() },
        Model::HS1
    );
    test_query!(
        test_query_firmware_version,
        "#5QF\r",
        "*5QF368\r",
        |mut driver: LSSDriver| async move { driver.query_firmware_version(5).await.unwrap() },
        "368".to_owned()
    );
    test_query!(
        test_query_serial_number,
        "#5QN\r",
        "*5QN12345678\r",
        |mut driver: LSSDriver| async move { driver.query_serial_number(5).await.unwrap() },
        "12345678".to_owned()
    );

    // Test blinking modes
    test_command!(
        test_blinking_mode_1,
        "#5CLB0\r",
        |mut driver: LSSDriver| async move {
            driver
                .set_led_blinking(5, vec![LedBlinking::NoBlinking])
                .await
                .unwrap()
        }
    );
    test_command!(
        test_blinking_mode_2,
        "#5CLB1\r",
        |mut driver: LSSDriver| async move {
            driver
                .set_led_blinking(5, vec![LedBlinking::Limp])
                .await
                .unwrap()
        }
    );
    test_command!(
        test_blinking_mode_3,
        "#5CLB2\r",
        |mut driver: LSSDriver| async move {
            driver
                .set_led_blinking(5, vec![LedBlinking::Holding])
                .await
                .unwrap()
        }
    );
    test_command!(
        test_blinking_mode_4,
        "#5CLB12\r",
        |mut driver: LSSDriver| async move {
            driver
                .set_led_blinking(
                    5,
                    vec![LedBlinking::Accelerating, LedBlinking::Decelerating],
                )
                .await
                .unwrap()
        }
    );
    test_command!(
        test_blinking_mode_5,
        "#5CLB48\r",
        |mut driver: LSSDriver| async move {
            driver
                .set_led_blinking(5, vec![LedBlinking::Free, LedBlinking::Travelling])
                .await
                .unwrap()
        }
    );
    test_command!(
        test_blinking_mode_6,
        "#5CLB63\r",
        |mut driver: LSSDriver| async move {
            driver
                .set_led_blinking(5, vec![LedBlinking::AlwaysBlink])
                .await
                .unwrap()
        }
    );

    test_command!(
        test_reset,
        "#254RESET\r",
        |mut driver: LSSDriver| async move { driver.reset(BROADCAST_ID).await.unwrap() }
    );

    test_query_float!(
        test_query_origin_offset,
        "#5QO\r",
        "*5QO-13\r",
        |mut driver: LSSDriver| async move { driver.query_origin_offset(5).await.unwrap() },
        -1.3
    );

    test_command!(
        test_set_origin_offset,
        "#5CO-24\r",
        |mut driver: LSSDriver| async move { driver.set_origin_offset(5, -2.4).await.unwrap() }
    );

    test_query_float!(
        test_query_angular_range,
        "#5QAR\r",
        "*5QAR1800\r",
        |mut driver: LSSDriver| async move { driver.query_angular_range(5).await.unwrap() },
        180.0
    );

    test_command!(
        test_set_angular_range,
        "#5CAR1800\r",
        |mut driver: LSSDriver| async move { driver.set_angular_range(5, 180.0).await.unwrap() }
    );

    test_query!(
        test_query_pwm_position,
        "#5QP\r",
        "*5QP2334\r",
        |mut driver: LSSDriver| async move { driver.query_pwm_position(5).await.unwrap() },
        2334
    );

    test_command!(
        test_move_to_pwm_position,
        "#5P2334\r",
        |mut driver: LSSDriver| async move { driver.move_to_pwm_position(5, 2334).await.unwrap() }
    );

    test_command!(
        test_move_to_pwm_position_with_modifier,
        "#5P2334S750\r",
        |mut driver: LSSDriver| async move {
            driver
                .move_to_pwm_position_with_modifier(5, 2334, CommandModifier::Speed(750))
                .await
                .unwrap()
        }
    );

    test_command!(
        test_move_to_pwm_position_with_modifiers,
        "#5P2334S750T2500\r",
        |mut driver: LSSDriver| async move {
            driver
                .move_to_pwm_position_with_modifiers(
                    5,
                    2334,
                    &[CommandModifier::Speed(750), CommandModifier::Timed(2500)],
                )
                .await
                .unwrap()
        }
    );
}
