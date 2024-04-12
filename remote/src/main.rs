#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::future::pending;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

extern crate sx127x_lora;

use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    embassy,
    peripherals::Peripherals,
    prelude::*,
    spi::{master::Spi, SpiMode},
    Delay, IO,
};

mod radio;

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
    let delay = Delay::new(&clocks);

    embassy::init(
        &clocks,
        hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOG_LEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Pins for Radio
    let reset = io.pins.gpio3.into_push_pull_output();
    let d0 = io.pins.gpio4.into_floating_input();
    let nss = io.pins.gpio5.into_push_pull_output();

    // Pins for SPI
    let mosi = io.pins.gpio6;
    let miso = io.pins.gpio7;
    let sck = io.pins.gpio8;

    let spi = Spi::new(peripherals.SPI2, 25u32.kHz(), SpiMode::Mode0, &clocks)
        .with_sck(sck)
        .with_mosi(mosi)
        .with_miso(miso);

    let radio_task = radio::run(spi, reset, d0, nss, delay);

    spawner.must_spawn(radio_task);

    let () = pending().await;
    unreachable!()
}
