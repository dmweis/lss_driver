use iron_lss;
use clap::Clap;
use async_std::task::sleep;
use std::time::Duration;

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
    // Why does it exit too early if this is here? 
    // Send doesn't assure that data is actually sent
    // TODO: figure out how to assure that data is sent
    sleep(Duration::from_secs_f32(0.3)).await;
    Ok(())
}