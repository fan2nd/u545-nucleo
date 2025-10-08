#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // USER_BUTTON PC13
    let mut button = ExtiInput::new(p.PC13, p.EXTI13, embassy_stm32::gpio::Pull::Down);

    loop {
        button.wait_for_high().await;
        info!("button pushed!");
        button.wait_for_low().await;
        info!("button released!");
    }
}
