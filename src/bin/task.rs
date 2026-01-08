#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _ = embassy_stm32::init(Default::default());
    info!("Hello World!");
    for i in 1..=3 {
        spawner.spawn(hello(i).unwrap());
    }
}

#[embassy_executor::task(pool_size = 3)]
async fn hello(delay: usize) {
    loop {
        Timer::after_secs(delay as u64).await;
        info!("hello from task {}", delay);
    }
}
