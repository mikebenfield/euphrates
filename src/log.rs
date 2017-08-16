//! # Logging facilities

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
        if DO_LOG_MAJOR {
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
        if DO_LOG_FAULT {
            println!(
                "Fault: {}",
                format!($fmt $(, $arg)*)
            )
        }
    }
}
