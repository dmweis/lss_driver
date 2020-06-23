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

fn main() {
    let args: Args = Args::parse();
    let mut driver = iron_lss::LSSDriver::new(&args.port).unwrap();
    println!("Voltage is {}V", driver.read_voltage(5).unwrap());
}
