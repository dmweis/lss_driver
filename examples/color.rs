use async_std::task::sleep;
use clap::Clap;
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
    let colors = vec![
        lss_driver::LedColor::Off,
        lss_driver::LedColor::Red,
        lss_driver::LedColor::Green,
        lss_driver::LedColor::Blue,
        lss_driver::LedColor::Yellow,
        lss_driver::LedColor::Cyan,
        lss_driver::LedColor::Magenta,
        lss_driver::LedColor::White,
    ];
    let mut driver = lss_driver::LSSDriver::new(&args.port).unwrap();
    loop {
        for color in &colors {
            driver.set_color(lss_driver::BROADCAST_ID, *color).await?;
            sleep(Duration::from_secs_f32(0.02)).await;
        }
    }
}
