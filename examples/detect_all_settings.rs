use clap::Parser;

#[derive(Parser)]
#[clap()]
struct Args {
    #[clap(about = "Serial port to use")]
    port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let mut driver = lss_driver::LSSDriver::new(&args.port)?;
    for i in 0..254 {
        if driver.query_status(i).await.is_ok() {
            println!("servo_id: {}", i);
            println!(
                "  firmware_version: {}",
                driver.query_firmware_version(i).await?
            );
            println!("  model: {:?}", driver.query_model(i).await?);
            println!("  serial_number: {}", driver.query_serial_number(i).await?);
            println!("  status: {:?}", driver.query_status(i).await?);
            println!(
                "  safety_status: {:?}",
                driver.query_safety_status(i).await?
            );
            println!(
                "  motion_profile: {}",
                driver.query_motion_profile(i).await?
            );
            println!(
                "  angular_acceleration: {}",
                driver.query_angular_acceleration(i).await?
            );
            println!(
                "  angular_deceleration: {}",
                driver.query_angular_deceleration(i).await?
            );
            println!(
                "  filter_position_count: {}",
                driver.query_filter_position_count(i).await?
            );
            println!(
                "  angular_holding_stiffness: {}",
                driver.query_angular_holding_stiffness(i).await?
            );
            println!(
                "  angular_stiffness: {}",
                driver.query_angular_stiffness(i).await?
            );
            println!("  maximum_speed: {}", driver.query_maximum_speed(i).await?);
        }
    }
    Ok(())
}
