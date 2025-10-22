#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[derive(defmt::Format, PartialEq, Eq)]
enum Event {
    Run,
    Stop,
}
#[derive(defmt::Format, PartialEq, Eq, Clone, Copy)]
enum State {
    Idle,
    Running,
}

struct SM {
    current: State,
    future: Option<Timer>,
}

impl SM {
    fn process(&mut self, e: Event) {
        match (self.current, e) {
            (State::Idle, Event::Run) => {
                self.current = State::Running;
                self.future = Some(Timer::after_secs(5));
                info!("entry: {}", self.current);
            }
            (State::Running, Event::Stop) => {
                self.current = State::Idle;
                self.future = None;
                info!("entry: {}", self.current);
            }
            _ => {}
        }
    }
}

static CHANNEL: Channel<ThreadModeRawMutex, Event, 3> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _ = embassy_stm32::init(Default::default());
    info!("Hello World!");
    spawner.spawn(receive().unwrap());
    spawner.spawn(send().unwrap());
}

#[embassy_executor::task]
async fn receive() {
    let mut state: SM = SM {
        current: State::Idle,
        future: None,
    };

    loop {
        if let Some(timer) = state.future.take() {
            // you can use future_utils::select to keep `timer`
            match select(timer, CHANNEL.receive()).await {
                Either::First(()) => {
                    info!("timer timeout")
                }
                Either::Second(e) => {
                    state.process(e);
                }
            }
        } else {
            state.process(CHANNEL.receiver().receive().await);
        }
    }
}

#[embassy_executor::task]
async fn send() {
    loop {
        CHANNEL.sender().send(Event::Run).await;
        Timer::after_secs(1).await;
        CHANNEL.sender().send(Event::Stop).await;
        Timer::after_secs(1).await;
    }
}
