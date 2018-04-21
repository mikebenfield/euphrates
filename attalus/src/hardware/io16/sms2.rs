use std::convert::{AsMut, AsRef};

use hardware::irq::Irq;
use hardware::sms_vdp::SmsVdpInternal;
use hardware::sn76489;
// use memo::{Inbox, Memo, Payload};
use super::Io16Impler;

// pub mod manifests {
//     use memo::{Item, Manifest, PayloadType};

//     pub const DEVICE_STRING: &'static str = &"Sms2Io";

//     const CONTENTS: &'static [Item] = &[
//         Item {
//             hex: true,
//             description: "address",
//         },
//         Item {
//             hex: true,
//             description: "value",
//         },
//     ];

//     pub const INPUT: &'static Manifest = &Manifest {
//         device: DEVICE_STRING,
//         summary: &"Input",
//         payload_type: PayloadType::U16,
//         contents: CONTENTS,
//     };

//     pub const OUTPUT: &'static Manifest = &Manifest {
//         device: DEVICE_STRING,
//         summary: &"Output to address {:0} with value {:1}",
//         payload_type: PayloadType::U16,
//         contents: CONTENTS,
//     };

//     pub const BOGUS_OUTPUT: &'static Manifest = &Manifest {
//         device: DEVICE_STRING,
//         summary: &"Bogus output to address {:0} with value {:1}",
//         payload_type: PayloadType::U16,
//         contents: CONTENTS,
//     };
// }

/// The IO system in the Sega Master Sytem 2.
///
/// It's almost identical to that in the original Sega Master System, but a
/// little simpler to implement.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Sms2Io {
    memory_control: u8,
    io_control: u8,
    id: u32,
    joypad_a: u8,
    joypad_b: u8,
    pause: bool,
}

impl Default for Sms2Io {
    fn default() -> Self {
        Sms2Io {
            memory_control: 0,
            io_control: 0,
            id: 0,
            joypad_a: 0xFF,
            joypad_b: 0xFF,
            pause: false,
        }
    }
}

impl Sms2Io {
    pub fn new() -> Sms2Io {
        Default::default()
    }

    #[inline]
    pub fn joypad_a(&self) -> u8 {
        self.joypad_a
    }

    #[inline]
    pub fn set_joypad_a(&mut self, x: u8) {
        self.joypad_a = x;
    }

    #[inline]
    pub fn joypad_b(&self) -> u8 {
        self.joypad_b
    }

    #[inline]
    pub fn set_joypad_b(&mut self, x: u8) {
        self.joypad_b = x;
    }

    #[inline]
    pub fn pause(&self) -> bool {
        self.pause
    }

    #[inline]
    pub fn set_pause(&mut self, x: bool) {
        self.pause = x;
    }
}

impl Irq for Sms2Io {
    #[inline]
    fn requesting_nmi(&self) -> bool {
        self.pause
    }

    #[inline]
    fn clear_nmi(&mut self) {
        self.pause = false
    }
}

impl<S> Io16Impler<S> for Sms2Io
where
    S: AsMut<Sms2Io> + AsRef<Sms2Io> + SmsVdpInternal + sn76489::Sn76489Internal + ?Sized,
{
    fn input(s: &mut S, address: u16) -> u8 {
        let masked = (address & 0b11000001) as u8;
        let value = match masked {
            0b00000000 => {
                // This is what the SMS 2 does. In the original SMS, reads
                // give the last byte of the instruction which read the
                // port. I'm not implementing that for now or hopefully
                // ever.
                0xFF
            }
            0b00000001 => {
                // ditto
                0xFF
            }
            0b01000000 => {
                // V counter
                s.read_v()
            }
            0b01000001 => {
                // H counter
                s.read_h()
            }
            0b10000000 => {
                // VDP data
                s.read_data()
            }
            0b10000001 => {
                // VDP control
                s.read_control()
            }
            0b11000000 => {
                // IO port A/B register
                AsRef::<Sms2Io>::as_ref(s).joypad_a()
            }
            0b11000001 => {
                // IO port B register
                AsRef::<Sms2Io>::as_ref(s).joypad_b()
            }
            _ => {
                unreachable!("Missing IO address in input");
            }
        };

        // s.receive(Memo::new(
        //     Payload::U16([address, value as u16, 0, 0]),
        //     manifests::INPUT,
        // ));

        value
    }

    fn output(s: &mut S, address: u16, value: u8) {
        // s.receive(Memo::new(
        //     Payload::U16([address, value as u16, 0, 0]),
        //     manifests::OUTPUT,
        // ));

        let masked = (address & 0b11000001) as u8;

        match masked {
            0b00000000 => {
                AsMut::<Sms2Io>::as_mut(s).memory_control = value;
            }
            0b00000001 => {
                AsMut::<Sms2Io>::as_mut(s).io_control = value;
            }
            0b01000000 => s.write(value),
            0b01000001 => s.write(value),
            0b10000000 => {
                s.write_data(value);
            }
            0b10000001 => {
                s.write_control(value);
            }
            _ => {
                // writes to the remaining addresses have no effect
                // s.receive(Memo::new(
                //     Payload::U16([address, value as u16, 0, 0]),
                //     manifests::BOGUS_OUTPUT,
                // ));
            }
        }
    }
}
