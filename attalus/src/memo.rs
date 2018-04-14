/// Memos - simple messages sent from devices to the user
///
/// Memos are useful for debugging.

use std::fmt::{self, Write};
use std::mem::transmute;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Payload {
    U8([u8; 8]),
    U16([u16; 4]),
    U32([u32; 2]),
    U64([u64; 1]),
    I8([i8; 8]),
    I16([i16; 4]),
    I32([i32; 2]),
    I64([i64; 1]),
    F32([f32; 2]),
    F64([f64; 1]),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PayloadType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

#[derive(Clone, Debug)]
pub enum Descriptions {
    Strings(&'static [&'static str]),
    Function(fn(u64) -> String),
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub device: &'static str,
    pub summary: &'static str,
    pub descriptions: Descriptions,
    pub payload_type: PayloadType,
}

impl Manifest {
    #[inline]
    pub fn send<I: Inbox>(&'static self, inbox: &mut I, payload: Payload) {
        let memo = Memo::new(payload, self);
        inbox.receive(memo);
    }
}

impl PartialEq<Self> for Manifest {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl Eq for Manifest {}

impl Hash for Manifest {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_usize(self as *const _ as *const u8 as usize);
    }
}

/// A message sent from a device to the user.
#[derive(Hash, Eq, PartialEq)]
pub struct Memo {
    payload: u64,
    manifest: &'static Manifest,
}

impl Memo {
    /// Create a memo.
    ///
    /// Will panic if the type of the Payload and payload_type in the `manifest`
    /// don't correspond.

    // inline must be available so that it can be compiled away when Memos are
    // ignored
    #[inline]
    pub fn new(payload: Payload, manifest: &'static Manifest) -> Memo {
        let payload2 = match (payload, manifest.payload_type) {
            (Payload::U8(x), PayloadType::U8) => unsafe { transmute(x) },
            (Payload::U16(x), PayloadType::U16) => unsafe { transmute(x) },
            (Payload::U32(x), PayloadType::U32) => unsafe { transmute(x) },
            (Payload::U64(x), PayloadType::U64) => unsafe { transmute(x) },
            (Payload::I8(x), PayloadType::I8) => unsafe { transmute(x) },
            (Payload::I16(x), PayloadType::I16) => unsafe { transmute(x) },
            (Payload::I32(x), PayloadType::I32) => unsafe { transmute(x) },
            (Payload::I64(x), PayloadType::I64) => unsafe { transmute(x) },
            (Payload::F64(x), PayloadType::F64) => unsafe { transmute(x) },
            (Payload::F32(x), PayloadType::F32) => unsafe { transmute(x) },
            _ => panic!("Payload and PayloadType must match"),
        };

        Memo {
            manifest,
            payload: payload2,
        }
    }

    // inline must be available so that it can be compiled away when Memos are
    // ignored
    #[inline]
    pub fn payload(&self) -> Payload {
        match self.manifest.payload_type {
            PayloadType::U8 => Payload::U8(unsafe { transmute(self.payload) }),
            PayloadType::U16 => Payload::U16(unsafe { transmute(self.payload) }),
            PayloadType::U32 => Payload::U32(unsafe { transmute(self.payload) }),
            PayloadType::U64 => Payload::U64(unsafe { transmute(self.payload) }),
            PayloadType::I8 => Payload::I8(unsafe { transmute(self.payload) }),
            PayloadType::I16 => Payload::I16(unsafe { transmute(self.payload) }),
            PayloadType::I32 => Payload::I32(unsafe { transmute(self.payload) }),
            PayloadType::I64 => Payload::I64(unsafe { transmute(self.payload) }),
            PayloadType::F32 => Payload::F32(unsafe { transmute(self.payload) }),
            PayloadType::F64 => Payload::F64(unsafe { transmute(self.payload) }),
        }
    }
}

impl fmt::Display for Memo {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let manifest = self.manifest;

        let s = format!("{:10}: {}.", manifest.device, manifest.summary);

        fn format_items<T: Copy, F: Fn(T) -> String>(
            display: F,
            items: &[T],
            descriptions: &'static [&'static str],
        ) -> String {
            let mut s = if descriptions.len() != 0 {
                format!(" -- {}: {}", descriptions[0], display(items[0]))
            } else {
                String::new()
            };

            for (item, description) in items[1..].iter().zip(descriptions[1..].iter()) {
                write!(s, ", {}: {}", display(*item), description).unwrap();
            }

            return s;
        }

        let s2 = match manifest.descriptions {
            Descriptions::Function(f) => f(self.payload),
            Descriptions::Strings(descriptions) => match self.payload() {
                Payload::U8(ref x) => format_items(|i| format!("{:0>2}", i), x, descriptions),
                Payload::U16(ref x) => format_items(|i| format!("{:0>4}", i), x, descriptions),
                Payload::U32(ref x) => format_items(|i| format!("{:0>8}", i), x, descriptions),
                Payload::U64(ref x) => {
                    format_items(|i| format!("{:0>width$}", i, width = 16), x, descriptions)
                }
                Payload::I8(ref x) => format_items(|i| format!("{:0>+2}", i), x, descriptions),
                Payload::I16(ref x) => format_items(|i| format!("{:0>+4}", i), x, descriptions),
                Payload::I32(ref x) => format_items(|i| format!("{:0>+8}", i), x, descriptions),
                Payload::I64(ref x) => {
                    format_items(|i| format!("{:0>+width$}", i, width = 16), x, descriptions)
                }
                Payload::F32(ref x) => format_items(|i| format!("{: >+8.5}", i), x, descriptions),
                Payload::F64(ref x) => format_items(|i| format!("{: >+8.5}", i), x, descriptions),
            },
        };

        let result = format!("{}{}", s, s2);

        f.pad(&result)
    }
}

/// A type that can be paused.
///
/// The intent is that this is used for a debugger that may want to pause
/// emulation. It's here in the `memo` module because that will often be in
/// response to a certain memo.
pub trait Pausable {
    #[inline]
    fn wants_pause(&self) -> bool {
        false
    }

    #[inline]
    fn clear_pause(&mut self) {}
}

pub trait PausableImpler<S>
where
    S: ?Sized,
{
    fn wants_pause(&S) -> bool;
    fn clear_pause(&mut S);
}

pub trait PausableImpl {
    type Impler: PausableImpler<Self>;
}

impl<S> Pausable for S
where
    S: PausableImpl,
{
    #[inline]
    fn wants_pause(&self) -> bool {
        <S as PausableImpl>::Impler::wants_pause(self)
    }

    #[inline]
    fn clear_pause(&mut self) {
        <S as PausableImpl>::Impler::clear_pause(self)
    }
}

pub trait Inbox: Pausable {
    fn receive(&mut self, memo: Memo);
}

pub trait InboxImpler<S>
where
    S: ?Sized,
{
    fn receive(&mut S, memo: Memo);
}

pub trait InboxImpl {
    type Impler: InboxImpler<Self>;
}

impl<S> Inbox for S
where
    S: Pausable + InboxImpl,
{
    #[inline]
    fn receive(&mut self, memo: Memo) {
        <S as InboxImpl>::Impler::receive(self, memo)
    }
}

/// An Inbox that throws away its memos.
///
/// You can also use it as an `InboxImpler` or a `PausableImpler` if you
/// want to do nothing.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NothingInbox;

impl Pausable for NothingInbox {
    #[inline]
    fn wants_pause(&self) -> bool {
        false
    }

    #[inline]
    fn clear_pause(&mut self) {}
}

impl<S> PausableImpler<S> for NothingInbox {
    #[inline]
    fn wants_pause(_: &S) -> bool {
        false
    }

    #[inline]
    fn clear_pause(_: &mut S) {}
}

impl Inbox for NothingInbox {
    #[inline]
    fn receive(&mut self, _memo: Memo) {}
}

impl<S> InboxImpler<S> for NothingInbox {
    #[inline]
    fn receive(_: &mut S, _memo: Memo) {}
}
