#![no_std]

use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Message {
    header: Header,
    body: Body,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Header {
    id: u32,
    version: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum Body {
    // Tells the vehicle to go forward or backward.
    // [0; 126] -> Backward. 0 is full speed backward.
    // 127 -> Neutral
    // [128; 255] -> Forward. 255 is full speed forward.
    Gas(u8),
    // Tells the vehicle to go Left or Right.
    // [0; 126] -> Left. 0 is full speed left.
    // 127 -> Neutral
    // [128; 255] -> Right. 255 is full speed right.
    Rotation(u8),
    // Mode the must turn in.
    Mode(Mode),
    Heartbeat,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum Mode {
    // Turn off the vehicle.
    Off,
    // Vehicle is controlled by the user with controller.
    Manual,
    // Vehicle is returning to the controller.
    Rth,
}

const MAX_ENCODE_SIZE: usize = 255;
pub type Buffer = String<MAX_ENCODE_SIZE>;

impl Message {
    // const MAX_ENCODE_SIZE: usize = 255;
    // pub type Buffer = String<{ Self::MAX_ENCODE_SIZE }>;

    pub fn new(id: u32, body: Body) -> Message {
        const VERSION: u16 = 0;

        Message {
            header: Header {
                id,
                version: VERSION,
            },
            body,
        }
    }

    pub fn encode(&self) -> Result<Buffer, serde_json_core::ser::Error> {
        serde_json_core::to_string(self)
    }

    pub fn decode(data: Buffer) -> Result<(Self, usize), serde_json_core::de::Error> {
        serde_json_core::from_str(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_decode(message: Message) {
        let encoded = message.encode();
        assert!(encoded.is_ok());

        let encoded = encoded.unwrap();
        let encoded_len = encoded.len();

        let decoded = Message::decode(encoded);
        assert!(decoded.is_ok());
        let (decoded, decoded_len) = decoded.unwrap();

        assert_eq!(encoded_len, decoded_len);
        assert_eq!(message, decoded);
    }

    #[test]
    fn encode_and_decode_all_bodies() {
        encode_decode(Message::new(1, Body::Gas(127)));
        encode_decode(Message::new(2, Body::Rotation(0)));
        encode_decode(Message::new(4, Body::Mode(Mode::Rth)));
    }
}
