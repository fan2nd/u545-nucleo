#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::OutputType,
    time::Hertz,
    timer::{
        Channel,
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        low_level::CountingMode,
    },
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // LED2 PA5
    let pwmpin = ComplementaryPwmPin::new(p.PA5, OutputType::PushPull);
    let mut pwm = ComplementaryPwm::new(
        p.TIM8,
        None,
        Some(pwmpin),
        None,
        None,
        None,
        None,
        None,
        None,
        Hertz::hz(1000),
        CountingMode::EdgeAlignedUp,
    );
    pwm.enable(Channel::Ch1);
    let duty = pwm.get_max_duty();
    loop {
        for i in 0..=10 {
            pwm.set_duty(Channel::Ch1, duty / 10 * i);
            Timer::after_secs(1).await;
        }
    }
}
