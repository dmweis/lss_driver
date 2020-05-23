use serialport;
use std::error::Error;

#[derive(Copy, Clone)]
enum LedColor {
    Off = 0,
    Red = 1,
    Green = 2,
    Blue = 3,
    Yellow = 4,
    Cyan = 5,
    Magenta = 6,
    White = 7,
}

struct LSSDriver {
    port: Box<dyn serialport::SerialPort>,
}

impl LSSDriver {
    fn new(port: &str) -> Result<LSSDriver, Box<dyn Error>> {
        let mut settings = serialport::SerialPortSettings::default();
        settings.baud_rate = 115200;
        let serial_port = serialport::open_with_settings(port, &settings)?;
        Ok(LSSDriver {
            port: serial_port
        })
    }

    fn set_color(&mut self, id: u8, color: LedColor) -> Result<(), Box<dyn Error>> {
        let message = format!("#{}LED{}\r", id, color as u8);
        let bytes = message.as_bytes();
        self.port.write_all(bytes)?;
        Ok(())
    }
}

fn main() {
    println!("Hello, world!");
}
