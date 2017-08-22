// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub const DO_LOG_MINOR: bool = true;
pub const DO_LOG_MAJOR: bool = true;
pub const DO_LOG_FAULT: bool = true;

#[macro_export]
macro_rules! log_minor {
    ($fmt: expr $(, $arg: expr)*) => {
        if log::DO_LOG_MINOR {
            println!(
                "Minor: {}",
                format!($fmt $(, $arg)*)
            )
        }
    }
}

#[macro_export]
macro_rules! log_major {
    ($fmt: expr $(, $arg: expr)*) => {
        if log::DO_LOG_MAJOR {
            println!(
                "Major: {}",
                format!($fmt $(, $arg)*)
            )
        }
    }
}

#[macro_export]
macro_rules! log_fault {
    ($fmt: expr $(, $arg: expr)*) => {
        if log::DO_LOG_FAULT {
            println!(
                "Fault: {}",
                format!($fmt $(, $arg)*)
            )
        }
    }
}
