use iron_lss;
use looprate::{ Rate, RateTimer };
use std::time::Instant;
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
    let mut rate = Rate::from_frequency(100.0);
    let mut loop_rate_counter = RateTimer::new();
    let mut driver = iron_lss::LSSDriver::new(&args.port).unwrap();
    driver.set_color(5, iron_lss::LedColor::Green).await?;
    driver.disable_motion_profile(5, false).await?;
    loop {
        rate.wait();
        driver.move_to_position(5, (start.elapsed().as_secs_f32()).sin() * 90.0).await?;
        loop_rate_counter.tick();
    }
}