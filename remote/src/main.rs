#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::future::pending;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use hal::{clock::ClockControl, embassy, peripherals::Peripherals, prelude::*};

#[embassy_executor::task]
async fn basic_task() {
    loop {
        log::info!("Hello from the embassy task");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // embassy::init(
    //     &clocks,
    //     hal::timer::TimerGroup::new(peripherals.TIMG0, &clocks),
    // );

    embassy::init(
        &clocks,
        hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    spawner.must_spawn(basic_task());

    let () = pending().await;
    unreachable!()
}
