use clap::Clap;

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
    for i in 0..254 {
        if driver.query_status(i).await.is_ok() {
            println!("Found servo with ID {}", i);
        }
    }
    Ok(())
}
