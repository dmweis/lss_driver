use looprate::{ Rate, RateTimer };
use std::time::Instant;


fn main() {
    let start = Instant::now();
    let mut rate = Rate::from_frequency(100.0);
    let mut loop_rate_counter = RateTimer::new();
    let mut driver = LSSDriver::new("COM14").unwrap();
    driver.set_color(5, LedColor::Green).unwrap();
    driver.disable_motion_profile(5, true).unwrap();
    loop {
        rate.wait();
        driver.move_to_position(5, (start.elapsed().as_secs_f32()).sin() * 90.0).unwrap();
        loop_rate_counter.tick();
    }
}