// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use serde::ser::{Serialize};
use serde::de::{DeserializeOwned};

pub trait Outbox
{
    type Memo: DeserializeOwned + Serialize;

    fn id(&self) -> u32;

    fn set_id(&mut self, id: u32);
}

pub trait Pausable {
    fn wants_pause(&self) -> bool;

    fn clear_pause(&mut self);
}

pub trait Inbox<M>: Pausable {
    fn receive(&mut self, id: u32, memo: M);
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NothingInbox;

impl Pausable for NothingInbox {
    #[inline(always)]
    fn wants_pause(&self) -> bool { false }

    #[inline(always)]
    fn clear_pause(&mut self) {}
}

impl<M> Inbox<M> for NothingInbox {
    #[inline(always)]
    fn receive(&mut self, _id: u32, _memo: M) {}
}
