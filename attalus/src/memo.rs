/// Memos - simple messages sent from devices to the user
///
/// Memos are useful for debugging.

use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum PayloadType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl PayloadType {
    /// How many bytes are in each entity in such a payload?
    pub fn size(self) -> usize {
        match self {
            PayloadType::U8 => 1,
            PayloadType::U16 => 2,
            PayloadType::U32 => 4,
            PayloadType::U64 => 8,
            PayloadType::I8 => 1,
            PayloadType::I16 => 2,
            PayloadType::I32 => 4,
            PayloadType::I64 => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Payload {
    U8([u8; 8]),
    U16([u16; 4]),
    U32([u32; 2]),
    U64(u64),
    I8([i8; 8]),
    I16([i16; 4]),
    I32([i32; 2]),
    I64(i64),
}

impl Payload {
    pub fn typ(self) -> PayloadType {
        match self {
            Payload::U8(_) => PayloadType::U8,
            Payload::U16(_) => PayloadType::U16,
            Payload::U32(_) => PayloadType::U32,
            Payload::U64(_) => PayloadType::U64,
            Payload::I8(_) => PayloadType::I8,
            Payload::I16(_) => PayloadType::I16,
            Payload::I32(_) => PayloadType::I32,
            Payload::I64(_) => PayloadType::I64,
        }
    }
}

/// A description or type of a Memo.
///
/// For instance, if a device wants to send memos for memory reads, all its
/// memory reads should have the same `Manifest`.
///
/// Note that equality for `Manifest`s is pointer equality, because it is
/// intended that they be compared quickly to see if two `Memo`s are the same
/// type, and they are stored as static references in `Memo`s.
#[derive(Debug)]
pub struct Manifest {
    pub summary: &'static str,
    pub device: &'static str,

    // saves memory to have payload_type as part of the manifest, instead of
    // having Memo have a Payload
    pub payload_type: PayloadType,
    pub content: &'static [&'static str],
}

impl Hash for Manifest {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_usize(self as *const _ as usize)
    }
}

impl PartialEq<Manifest> for Manifest {
    #[inline]
    fn eq(&self, other: &Manifest) -> bool {
        self as *const _ == other as *const _
    }
}

impl Eq for Manifest {}

/// A message sent from a device to the user.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Memo {
    payload: [u8; 8],
    manifest: &'static Manifest,
}

impl Memo {
    /// Create a memo.
    ///
    /// Will panic if the type of the Payload and payload_type in the `manifest`
    /// don't correspond.
    #[inline]
    pub fn new(payload: Payload, manifest: &'static Manifest) -> Memo {
        use std::mem::transmute;

        let payload2 = match (payload, manifest.payload_type) {
            (Payload::U8(x), PayloadType::U8) => unsafe { transmute(x) },
            (Payload::U16(x), PayloadType::U16) => unsafe { transmute(x) },
            (Payload::U32(x), PayloadType::U32) => unsafe { transmute(x) },
            (Payload::U64(x), PayloadType::U64) => unsafe { transmute(x) },
            (Payload::I8(x), PayloadType::I8) => unsafe { transmute(x) },
            (Payload::I16(x), PayloadType::I16) => unsafe { transmute(x) },
            (Payload::I32(x), PayloadType::I32) => unsafe { transmute(x) },
            (Payload::I64(x), PayloadType::I64) => unsafe { transmute(x) },
            _ => panic!("Payload and PayloadType must match"),
        };

        Memo {
            manifest,
            payload: payload2,
        }
    }

    pub fn payload(&self) -> Payload {
        use std::mem::transmute;

        match self.manifest.payload_type {
            PayloadType::U8 => Payload::U8(unsafe { transmute(self.payload) }),
            PayloadType::U16 => Payload::U16(unsafe { transmute(self.payload) }),
            PayloadType::U32 => Payload::U32(unsafe { transmute(self.payload) }),
            PayloadType::U64 => Payload::U64(unsafe { transmute(self.payload) }),
            PayloadType::I8 => Payload::I8(unsafe { transmute(self.payload) }),
            PayloadType::I16 => Payload::I16(unsafe { transmute(self.payload) }),
            PayloadType::I32 => Payload::I32(unsafe { transmute(self.payload) }),
            PayloadType::I64 => Payload::I64(unsafe { transmute(self.payload) }),
        }
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
