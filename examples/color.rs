use iron_lss;
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
    let colors = vec![
        iron_lss::LedColor::Off,
        iron_lss::LedColor::Red,
        iron_lss::LedColor::Green,
        iron_lss::LedColor::Blue,
        iron_lss::LedColor::Yellow,
        iron_lss::LedColor::Cyan,
        iron_lss::LedColor::Magenta,
        iron_lss::LedColor::White,
    ];
    let mut driver = iron_lss::LSSDriver::new(&args.port).unwrap();
    loop {
        for color in &colors {
            driver.set_color(5, *color).await?;
            sleep(Duration::from_secs_f32(0.3)).await;
        }
    }
}