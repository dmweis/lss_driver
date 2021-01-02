use async_std::task::sleep;
use clap::Clap;
use std::time::Duration;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(about = "Serial port to use")]
    port: String,
    #[clap(about = "ID you want to give to the connected servo", long = "id")]
    id: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let mut driver = lss_driver::LSSDriver::new(&args.port).unwrap();
    driver
        .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Red)
        .await?;
    driver.set_id(lss_driver::BROADCAST_ID, args.id).await?;
    driver.reset(lss_driver::BROADCAST_ID).await?;
    // After reset servo becomes unresponsive for a bit
    sleep(Duration::from_secs(2)).await;
    let new_id = driver.query_id(lss_driver::BROADCAST_ID).await?;
    if new_id == args.id {
        println!("ID set successfully");
        driver
            .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Green)
            .await?;
    } else {
        eprintln!("ID setting failed!");
    }
    Ok(())
}
