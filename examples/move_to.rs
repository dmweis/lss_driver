use clap::Clap;
use lss_driver;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(about = "Serial port to use")]
    port: String,
    #[clap(about = "Position to move to")]
    position: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let mut driver = lss_driver::LSSDriver::new(&args.port).unwrap();
    driver.move_to_position(5, args.position).await?;
    driver.set_color(5, lss_driver::LedColor::Magenta).await?;
    Ok(())
}
