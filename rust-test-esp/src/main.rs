#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::future::pending;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{Delay, embassy, prelude::*};
use esp_hal::clock::ClockControl;
use esp_hal::peripherals::Peripherals;
use esp_hal::mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, PeripheralClockConfig, MCPWM};
use esp_hal::gpio::{IO};

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
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pin = io.pins.gpio0;
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    //let mut delay = Delay::new(&clocks);
    spawner.must_spawn(basic_task());
    esp_println::logger::init_logger_from_env();
    embassy::init(
        &clocks,
        esp_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );
    #[cfg(feature = "esp32h2")]
        let clock_cfg = PeripheralClockConfig::with_frequency(&clocks, 40.MHz()).unwrap();
    #[cfg(not(feature = "esp32h2"))]
        let clock_cfg = PeripheralClockConfig::with_frequency(&clocks, esp_hal::prelude::_fugit_RateExtU32::MHz(32)).unwrap();
    let () = pending().await;
    let mut mcpwm = MCPWM::new(peripherals.MCPWM0, clock_cfg);
    // connect operator0 to timer0
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    // connect operator0 to pin
    let mut pwm_pin = mcpwm
        .operator0
        .with_pin_a(pin, PwmPinConfig::UP_ACTIVE_HIGH);
    // start timer with timestamp values in the range of 0..=99 and a frequency of
    // 20 kHz
    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, esp_hal::prelude::_fugit_RateExtU32::kHz(20))
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);
    // pin will be high 50% of the time
    pwm_pin.set_timestamp(50);
    unreachable!();
}
