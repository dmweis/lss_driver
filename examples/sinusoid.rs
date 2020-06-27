use lss_driver;
use std::time::Instant;
use async_std::task::sleep;
use std::time::Duration;
use clap::Clap;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(
        about = "Serial port to use"
    )]
    port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let start = Instant::now();
    let mut driver = lss_driver::LSSDriver::new(&args.port).unwrap();
    driver.set_color(5, lss_driver::LedColor::Green).await?;
    driver.set_motion_profile(5, false).await?;
    driver.set_angular_holding(5, -2).await?;
    driver.set_angular_stiffness(5, -2).await?;
    driver.set_filter_position_count(5, 4).await?;
    loop {
        driver.move_to_position(5, (start.elapsed().as_secs_f32() * 3.0).sin() * 90.0).await?;
        sleep(Duration::from_millis(20)).await;
    }
}