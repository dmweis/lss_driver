use async_std::task::sleep;
use clap::Clap;
use lss_driver;
use std::time::Duration;
use std::time::Instant;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(about = "Serial port to use")]
    port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let start = Instant::now();
    let mut driver = lss_driver::LSSDriver::new(&args.port).unwrap();
    driver.set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Green).await?;
    driver.set_motion_profile(lss_driver::BROADCAST_ID, false).await?;
    driver.set_angular_holding_stiffness(lss_driver::BROADCAST_ID, -2).await?;
    driver.set_angular_stiffness(lss_driver::BROADCAST_ID, -2).await?;
    driver.set_filter_position_count(lss_driver::BROADCAST_ID, 4).await?;
    loop {
        driver
            .move_to_position(lss_driver::BROADCAST_ID, (start.elapsed().as_secs_f32() * 3.0).sin() * 90.0)
            .await?;
        sleep(Duration::from_millis(20)).await;
    }
}
