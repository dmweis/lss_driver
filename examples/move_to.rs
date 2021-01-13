use clap::Clap;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(about = "Serial port to use")]
    port: String,
    #[clap(about = "Position to move to")]
    position: f32,
    #[clap(
        about = "ID of the motor you want to move. Default BROADCAST",
        long = "id",
        default_value = "254"
    )]
    id: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let mut driver = lss_driver::LSSDriver::new(&args.port).unwrap();
    driver.move_to_position(args.id, args.position).await?;
    driver
        .set_color(args.id, lss_driver::LedColor::Magenta)
        .await?;
    Ok(())
}
