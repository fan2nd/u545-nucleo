#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, peripherals,
    usart::{self, Config, Uart},
};
use embedded_cli::{Command, cli::CliBuilder};
use ufmt::uwriteln;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    let config = Config::default();
    let mut rx_buffer = [0u8; 32];
    let uart = Uart::new(
        p.USART1,
        p.PA10,
        p.PA9,
        Irqs,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        config,
    )
    .unwrap();
    let (mut tx, rx) = uart.split();
    let mut rx = rx.into_ring_buffered(&mut rx_buffer);
    let mut buffer = [0u8; 32];

    let mut cli = CliBuilder::default()
        .prompt("Rust> ")
        .command_buffer([0; 64])
        .history_buffer([0; 64])
        .writer(tx)
        .build()
        .unwrap();

    let mut buffer = [0u8; 32];
    loop {
        if let Ok(len) = rx.read(&mut buffer).await {
            buffer[0..len].iter().for_each(|&c| {
                cli.process_byte::<UserCommand, _>(
                    c,
                    &mut UserCommand::processor(|cli, cmd| {
                        match cmd {
                            UserCommand::Unsigned { num } => {
                                let writer = cli.writer();
                                uwriteln!(writer, "unsigned: {}", num).ok();
                            }
                            UserCommand::Signed { num } => {
                                let writer = cli.writer();
                                uwriteln!(writer, "signed: {}", num).ok();
                            }
                        };
                        Ok(())
                    }),
                )
                .ok();
            })
        }
    }
}

#[derive(Command)]
pub enum UserCommand {
    Unsigned { num: usize },
    Signed { num: isize },
}
