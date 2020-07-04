use async_std::task::sleep;
use clap::Clap;
use lss_driver;
use std::collections::HashMap;
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
    let mut driver = lss_driver::LSSDriver::new(&args.port)?;
    let mut ids = vec![];
    for i in 0..254 {
        match driver.query_status(i).await {
            Ok(_) => {
                ids.push(i);
                println!("Found servo with ID {}", i)
            }
            Err(_) => (),
        }
    }
    for id in &ids {
        driver.set_color(*id, lss_driver::LedColor::Magenta).await?;
    }
    loop {
        let mut frame_buffer = String::new();
        frame_buffer.push_str("\n");
        for id in &ids {
            let position = driver.query_position(*id).await?;
            frame_buffer.push_str(&format!("{} {}\n", id, position));
            if position > 0.0 {
                driver.set_color(*id, lss_driver::LedColor::Green).await?;
            } else {
                driver.set_color(*id, lss_driver::LedColor::Red).await?;
            }
        }
        print!("{}", frame_buffer);
        sleep(Duration::from_millis(500)).await;
    }
}
