use iron_lss;
use clap::Clap;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(
        about = "Serial port to use"
    )]
    port: String,
    #[clap(
        about = "Position to move to"
    )]
    position: f32,
}

fn main() {
    let args: Args = Args::parse();
    let mut driver = iron_lss::LSSDriver::new(&args.port).unwrap();
    driver.move_to_position(5, args.position).unwrap();
}