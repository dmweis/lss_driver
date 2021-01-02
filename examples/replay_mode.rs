use async_std::io;
use async_std::task::sleep;
use clap::Clap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(about = "Serial port to use")]
    port: String,
}

async fn wait_for_enter() -> Result<(), Box<dyn std::error::Error>> {
    println!("Press enter to continue...");
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.read_line(&mut line).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let mut driver = lss_driver::LSSDriver::new(&args.port)?;

    let running = Arc::new(AtomicBool::new(true));
    let running_handle = running.clone();

    ctrlc::set_handler(move || {
        running_handle.store(false, Ordering::SeqCst);
        println!("Caught interrupt");
    })?;

    let mut ids = vec![];
    for i in 0..254 {
        if driver.query_status(i).await.is_ok() {
            ids.push(i);
            println!("Found servo with ID {}", i)
        }
    }
    println!("Finished searching");
    driver
        .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Magenta)
        .await?;
    driver
        .set_motion_profile(lss_driver::BROADCAST_ID, false)
        .await?;
    driver
        .set_angular_holding_stiffness(lss_driver::BROADCAST_ID, -2)
        .await?;
    driver
        .set_angular_stiffness(lss_driver::BROADCAST_ID, -2)
        .await?;
    driver
        .set_filter_position_count(lss_driver::BROADCAST_ID, 4)
        .await?;
    driver.limp(lss_driver::BROADCAST_ID).await?;

    wait_for_enter().await?;
    println!("Starting recording");
    driver
        .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Red)
        .await?;
    let mut history = HashMap::new();
    for id in &ids {
        history.insert(id, vec![]);
    }

    let mut step_counter = 0;
    while running.load(Ordering::SeqCst) {
        for id in &ids {
            let position = driver.query_position(*id).await?;
            history.get_mut(id).unwrap().push(position);
        }
        step_counter += 1;
        sleep(Duration::from_millis(50)).await;
    }
    running.store(true, Ordering::SeqCst);
    println!("Finished recording!");
    driver
        .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Green)
        .await?;
    while running.load(Ordering::SeqCst) {
        driver
            .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Green)
            .await?;
        println!("Play backwards");
        wait_for_enter().await?;
        driver
            .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Cyan)
            .await?;
        for step in (0..step_counter).rev() {
            for (id, record) in &history {
                driver.move_to_position(**id, record[step]).await?;
            }
            sleep(Duration::from_millis(50)).await;
            if !running.load(Ordering::SeqCst) {
                break;
            }
        }
        if !running.load(Ordering::SeqCst) {
            break;
        }
        driver
            .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Green)
            .await?;
        println!("Play forwards");
        wait_for_enter().await?;
        driver
            .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Cyan)
            .await?;
        for step in 0..step_counter {
            for (id, record) in &history {
                driver.move_to_position(**id, record[step]).await?;
            }
            sleep(Duration::from_millis(50)).await;
            if !running.load(Ordering::SeqCst) {
                break;
            }
        }
    }
    driver
        .set_color(lss_driver::BROADCAST_ID, lss_driver::LedColor::Magenta)
        .await?;
    driver.limp(lss_driver::BROADCAST_ID).await?;
    sleep(Duration::from_millis(50)).await;
    Ok(())
}
