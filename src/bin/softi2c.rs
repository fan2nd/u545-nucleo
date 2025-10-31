#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use lm75::Lm75;
use u545_nucleo::softi2c::{SoftI2c, i2c_scan};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut i2c = SoftI2c::new(p.PC3, p.PC2, 400);
    let addrs = i2c_scan(&mut i2c);
    for addr in addrs.iter() {
        info!("0x{:x}", addr);
    }
    let mut lm75 = Lm75::new(i2c, 0x4f);
    let temp = lm75.read_temperature().unwrap();
    info!("temperature: {}", temp);
    loop {}
}
