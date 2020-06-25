use iron_lss;
use dynamixel_driver::DynamixelDriver;
use looprate::{ Rate, RateTimer };
use std::time::Instant;
use clap::Clap;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(
        about = "Lynxmotion serial port to use",
        short="l"
    )]
    lss: String,
    #[clap(
        about = "Dyanmixel serial port to use",
        short="d"
    )]
    dynamixel: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let start = Instant::now();
    let mut rate = Rate::from_frequency(200.0);
    let mut loop_rate_counter = RateTimer::new();
    let mut driver_lss = iron_lss::LSSDriver::new(&args.lss).unwrap();
    let mut driver_dyn = DynamixelDriver::new(&args.dynamixel).unwrap();

    driver_lss.set_color(5, iron_lss::LedColor::Green).await?;
    driver_lss.set_motion_profile(5, false).await?;
    loop {
        rate.wait();
        driver_lss.move_to_position(5, -(start.elapsed().as_secs_f32()).sin() * 90.0).await?;
        driver_dyn.write_position_degrees(2, (start.elapsed().as_secs_f32()).sin() * 90.0 + 150.0)?;
        loop_rate_counter.tick();
    }
}