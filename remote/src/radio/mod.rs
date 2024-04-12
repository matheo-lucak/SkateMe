use embassy_time::{Duration, Timer};
use hal::{
    peripherals::SPI2,
    spi::{master::Spi, FullDuplexMode},
    Delay,
};

pub mod pins {
    use hal::gpio::{Floating, GpioPin, Input, Output, PushPull};

    pub type Reset = GpioPin<Output<PushPull>, 3>;
    pub type D0 = GpioPin<Input<Floating>, 4>;
    pub type Nss = GpioPin<Output<PushPull>, 5>;
}

use sx127x_lora::LoRa;
const FREQUENCY: i64 = 433;

#[embassy_executor::task]
pub async fn run(
    spi: Spi<'static, SPI2, FullDuplexMode>,
    reset: pins::Reset,
    d0: pins::D0,
    nss: pins::Nss,
    delay: Delay,
) -> ! {
    let mut lora = sx127x_lora::LoRa::new(spi, nss, reset, FREQUENCY, delay)
        .expect("Failed to communicate with radio module!");

    lora.set_tx_power(17, 1).unwrap(); //Using PA_BOOST. See your board for correct pin.

    #[cfg(feature = "transmitter")]
    {
        transmitter(lora).await
    }
    #[cfg(not(feature = "transmitter"))]
    {
        receiver(lora, d0).await
    }
}

async fn transmitter(
    mut lora: LoRa<Spi<'static, SPI2, FullDuplexMode>, pins::Nss, pins::Reset, Delay>,
) -> ! {
    let mut message_counter: u32 = 0;

    loop {
        let message = protocol::Message::new(message_counter, protocol::Body::Heartbeat);

        let encoded = message.encode().unwrap();
        let encoded = encoded.as_bytes();
        let mut payload = [0; 255];
        payload[..encoded.len()].copy_from_slice(encoded);

        let transmit = lora.transmit_payload(payload, encoded.len());
        match transmit {
            Ok(()) => log::info!(
                "Sending message: {:?} with encoded size: {}",
                message,
                encoded.len()
            ),
            Err(e) => log::error!("Error: {:?}", e),
        };

        message_counter += 1;
        Timer::after(Duration::from_secs(5)).await;
    }
}

async fn receiver(
    mut lora: LoRa<Spi<'static, SPI2, FullDuplexMode>, pins::Nss, pins::Reset, Delay>,
    mut _d0: pins::D0,
) -> ! {
    loop {
        // d0.wait_for_falling_edge().await.unwrap();
        let packet_size = match lora.poll_irq(Some(5_000)) {
            Ok(size) => {
                log::info!("New packet polled with size {}", size);
                size
            }
            Err(_) => {
                log::error!("Timeout");
                continue;
            }
        };

        let buffer = match lora.read_packet() {
            Ok(buffer) => {
                let rssi = lora.get_packet_rssi().unwrap();
                log::info!("Received packet: {:?} with rssi {}", buffer, rssi);
                buffer
            }
            Err(_) => {
                log::error!("Error reading buffer");
                continue;
            }
        };

        let buffer = &buffer[0..packet_size];
        let buffer = protocol::Buffer::from_iter(buffer.iter().map(|item| char::from(*item)));

        let (message, size) = match protocol::Message::decode(buffer) {
            Ok(decoded) => decoded,
            Err(err) => {
                log::error!("Failed to decode message. Got error {err}");
                continue;
            }
        };

        log::info!("Message decoded {:?} with size {}", message, size);

        Timer::after(Duration::from_millis(100)).await;
    }
}
