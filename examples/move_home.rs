use iron_lss;
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
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args: Args = Args::parse();
    let mut driver = iron_lss::LSSDriver::new(&args.port).unwrap();
    driver.move_to_position(5, 0.0).await?;
    driver.set_color(5, iron_lss::LedColor::Magenta).await?;
    Ok(())
}