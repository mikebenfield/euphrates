use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub trait Outbox {
    type Memo: DeserializeOwned + Serialize;

    fn id(&self) -> u32;

    fn set_id(&mut self, id: u32);
}

pub trait Pausable {
    #[inline]
    fn wants_pause(&self) -> bool {
        false
    }

    #[inline]
    fn clear_pause(&mut self) {}
}

pub trait Inbox<M>: Pausable {
    fn receive(&mut self, id: u32, memo: M);
}

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

impl<M> Inbox<M> for NothingInbox {
    #[inline]
    fn receive(&mut self, _id: u32, _memo: M) {}
}
