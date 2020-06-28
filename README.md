# Lynxmotion Smart Servo Driver

[![Rust](https://github.com/dmweis/lss_driver/workflows/Rust/badge.svg)](https://github.com/dmweis/lss_driver/actions)  

This crate provides an asynchronous serial driver the the Lynxmotion smart servos.  

You can read more about the servos on the official [robotshop wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/)  

## Driver

The Smart servos are controlled over a serial UART protocol.  

You can read about the protocol [here](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/)
It's highly recommended to at least skim the protocol page before trying to use this driver.  

This driver currently doesn't implement all features from the protocol.  
Some missing features are modifiers and setup commands.  
If there are any missing commands or features you'd like added feel free to raise a PR or an issue.  

This driver uses `async/await`. As a result you will need to use an async runtime.
The driver is based on [tokio-serial](https://github.com/berkowski/tokio-serial) so tokio would be a good choice but any should work.

## Usage

This crate comes with multiple [examples](https://github.com/dmweis/lss_driver/tree/master/examples).  
These are a good start if you want to learn how to use it.  

```rust
use lss_driver;

#[tokio::main]
async fn main() {
    // Create a driver on port `COM14` or `/dev/ttyUSB0`...
    let mut driver = lss_driver::LSSDriver::new("COM14").unwrap();
    // In case there is only one servo connected
    // we can query it's ID using the broadcast ID
    let id = driver.query_id(lss_driver::BROADCAST_ID).await.unwrap();
    // move motor with ID 5 to 90.0 degrees
    driver.move_to_position(5, 90.0).await.unwrap();
    // Set color of servo with ID 5 to Magenta
    driver.set_color(5, lss_driver::LedColor::Magenta).await.unwrap();
}

```

## Building

This package shouldn't depend on any native libraries.  
Rust [serialport](https://gitlab.com/susurrus/serialport-rs) depends on `pkg-config` and `libudev-dev` on GNU Linux but they should be disabled for this crate.  
If you do run into issues with them failing it may be worth looking into their dependencies and raising an issue here.  
