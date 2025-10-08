#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, peripherals,
    usart::{self, BufferedUart, Config},
};
use embedded_io_async::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();
    let mut rx_buffer = [0u8; 32];
    let mut tx_buffer = [0u8; 32];
    let mut uart = BufferedUart::new(
        p.USART1,
        p.PA10,
        p.PA9,
        &mut tx_buffer,
        &mut rx_buffer,
        Irqs,
        config,
    )
    .unwrap();
    let mut buffer = [0u8; 32];
    loop {
        if let Ok(len) = uart.read(&mut buffer).await {
            info!("{}", &buffer[0..len]);
            uart.write_all(&buffer[0..len]).await.unwrap();
        }
    }
}
