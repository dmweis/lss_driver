use async_std::task::sleep;
// use std::thread::sleep;
use clap::Clap;
use ctrlc;
use lss_driver;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(about = "Serial port to use")]
    port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    let running = Arc::new(AtomicBool::new(true));
    let running_handle = running.clone();

    ctrlc::set_handler(move || {
        running_handle.store(false, Ordering::Release);
        println!("Ctrl-C detected. Terminating...")
    })
    .expect("Error setting Ctrl-C handler");

    let mut driver = lss_driver::LSSDriver::new(&args.port).unwrap();
    driver.set_color(5, lss_driver::LedColor::Green).await?;
    driver.set_motion_profile(5, true).await?;
    driver.set_angular_acceleration(5, 100).await?;
    driver.set_angular_deceleration(5, 80).await?;
    while running.load(Ordering::Acquire) {
        driver.move_to_position(5, 120.0).await?;
        driver.set_color(5, lss_driver::LedColor::Blue).await?;
        sleep(Duration::from_millis(50)).await;
        while driver.query_status(5).await? != lss_driver::MotorStatus::Holding {
            sleep(Duration::from_millis(20)).await;
        }
        if !running.load(Ordering::Acquire) {
            break;
        }
        driver.move_to_position(5, -120.0).await?;
        driver.set_color(5, lss_driver::LedColor::Red).await?;
        sleep(Duration::from_millis(50)).await;
        while driver.query_status(5).await? != lss_driver::MotorStatus::Holding {
            sleep(Duration::from_millis(20)).await;
        }
    }
    driver.limp(5).await?;
    driver.set_color(5, lss_driver::LedColor::Cyan).await?;
    sleep(Duration::from_millis(20)).await;
    Ok(())
}
