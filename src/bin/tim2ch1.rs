#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::OutputType,
    time::Hertz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // LED2 PA5
    let pwmpin = PwmPin::new(p.PA5, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM2,
        Some(pwmpin),
        None,
        None,
        None,
        Hertz::hz(1000),
        CountingMode::EdgeAlignedUp,
    );
    let mut led = pwm.ch1();
    led.enable();
    loop {
        for i in 0..=10 {
            led.set_duty_cycle_percent(i * 10);
            Timer::after_secs(1).await;
        }
    }
}
