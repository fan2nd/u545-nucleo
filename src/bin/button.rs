#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::{self, ExtiInput},
    interrupt,
};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs{
        EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // USER_BUTTON PC13
    let mut button = ExtiInput::new(p.PC13, p.EXTI13, embassy_stm32::gpio::Pull::Down, Irqs);

    loop {
        button.wait_for_high().await;
        info!("button pushed!");
        button.wait_for_low().await;
        info!("button released!");
    }
}
