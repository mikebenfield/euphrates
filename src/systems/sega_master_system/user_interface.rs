// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use super::*;

pub struct PlayerStatus {
    pub joypad_a: u8,
    pub joypad_b: u8,
}

pub trait UserInterface {
    fn update_player(&mut self);

    fn player_status(&self) -> PlayerStatus;

    fn update_user(&mut self, z: &mut MasterSystem);

    fn respond(&mut self, s: String);

    fn command(&mut self) -> Option<Command>;

    fn query(&mut self) -> Option<Query>;

    fn wants_quit(&self) -> bool;
}
