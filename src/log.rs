//! Facilities for the emulated SMS to record logs.
//!
//! Simple usage: create an instance of either the [`LogNothing`], [`LogErrors`],
//! [`LogMajorsAndErrors`], or [`LogEverything`] structs, then apply one of the
//! [`log_error`], [`log_major`], or [`log_minor`] macros.
//!
//! For example:
//! ```
//! let log = log::LogEverything(std::io::stdout());
//! log_minor(log, "Something minor happened");
//! log_major(log, "Something major happened");
//! log_error(log, "A tragic error happened");
//! ```
//! 
//! In increasing order of priority, there are minor, major, and error log
//! 
//! entries. The [`Log`] trait accepts all three of these, but is designed
//! so that `rustc` will optimize away unused
//!
//! [`Log`]: trait.Log.html
//! [`LogNothing`]: struct.LogNothing.html
//! [`LogErrors`]: struct.LogErrors.html
//! [`LogMajorsAndErrors`]: struct.LogMajorsAndErrors.html
//! [`LogEverything`]: struct.LogEverything.html
//! [`log_error`]: ../macro.log_error.html
//! [`log_major`]: ../macro.log_major.html
//! [`log_minor`]: ../macro.log_minor.html

use std;
use std::fmt;
use std::io::Write;

#[derive(Clone, Debug)]
pub struct Error {
    msg: String
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}

pub trait Log {
    fn log_minor0(&mut self, s: String);
    fn log_major0(&mut self, s: String);
    fn log_error0(&mut self, s: String);
    fn does_log_minor(&self) -> bool;
    fn does_log_major(&self) -> bool;
    fn does_log_error(&self) -> bool;
    fn check(&self) -> Result<(), Error>;
}

trait Bool {
    fn get() -> bool;
}

struct True;

impl Bool for True {
    fn get() -> bool { true }
}

struct False;

impl Bool for False {
    fn get() -> bool { false }
}

struct LogGeneral<LogMinor: Bool, LogMajor: Bool, LogError: Bool, W: Write> {
    write: W,
    error: Result<(), Error>,
    pd1: std::marker::PhantomData<LogMinor>,
    pd2: std::marker::PhantomData<LogMajor>,
    pd3: std::marker::PhantomData<LogError>,
}

impl<LogMinor: Bool, LogMajor: Bool, LogError: Bool, W: Write> Log for
  LogGeneral<LogMinor, LogMajor, LogError, W> {
    fn log_minor0(&mut self, s: String) {
        self.write.write_all(s.as_bytes());
        self.write.write_all(b"\n");
    }
    fn log_major0(&mut self, s: String) {
        self.write.write_all(s.as_bytes());
        self.write.write_all(b"\n");
    }
    fn log_error0(&mut self, s: String) {
        self.error =
            match &self.error {
                &Ok(()) => Err(Error { msg: s.clone() }),
                x => x.clone(),
            };
        self.write.write_all(s.as_bytes());
        self.write.write_all(b"\n");
    }
    fn does_log_minor(&self) -> bool { LogMinor::get() }
    fn does_log_major(&self) -> bool { LogMajor::get() }
    fn does_log_error(&self) -> bool { LogError::get() }
    fn check(&self) -> Result<(), Error> {
        self.error.clone()
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct WriteNothing;

impl Write for WriteNothing {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct LogNothing(LogGeneral<False, False, False, WriteNothing>);

impl LogNothing {
    pub fn new() -> LogNothing {
        LogNothing(
            LogGeneral {
                write: Default::default(),
                error: Ok(()),
                pd1: Default::default(),
                pd2: Default::default(),
                pd3: Default::default(),
            }
        )
    }
}

pub struct LogErrors<W: Write>(LogGeneral<False, False, True, W>);

pub struct LogMajorsAndErrors<W: Write>(LogGeneral<False, True, True, W>);

pub struct LogEverything<W: Write>(LogGeneral<True, True, True, W>);

macro_rules! impl_new {
    ($type_name: ident) => {
        impl<W: Write> $type_name<W> {
            pub fn new(write: W) -> $type_name<W> {
                $type_name(
                    LogGeneral {
                        write: write,
                        error: Ok(()),
                        pd1: Default::default(),
                        pd2: Default::default(),
                        pd3: Default::default(),
                    }
                )
            }
        }
    }
}

impl_new!{LogErrors}

impl_new!{LogMajorsAndErrors}

impl_new!{LogEverything}

macro_rules! impl_log {
    ([$($generic: tt)*] $($type_name: tt)*) => {
        impl $($generic)* Log for $($type_name)* {
            fn log_minor0(&mut self, s: String) {
                self.0.log_minor0(s)
            }
            fn log_major0(&mut self, s: String) {
                self.0.log_major0(s)
            }
            fn log_error0(&mut self, s: String) {
                self.0.log_error0(s)
            }
            fn does_log_minor(&self) -> bool {
                self.0.does_log_minor()
            }
            fn does_log_major(&self) -> bool {
                self.0.does_log_major()
            }
            fn does_log_error(&self) -> bool {
                self.0.does_log_error()
            }
            fn check(&self) -> Result<(), Error> {
                self.0.check()
            }
        }
    }
}

impl_log!{[] LogNothing}

impl_log!{[<W: Write>] LogErrors<W>}

impl_log!{[<W: Write>] LogMajorsAndErrors<W>}

impl_log!{[<W: Write>] LogEverything<W>}

#[macro_export]
macro_rules! log_minor {
    ($log: expr, $fmt: expr, $($arg: tt)*) => {
        if $log.does_log_minor() {
            $log.log_minor0(
                format!(
                    "Minor: {}",
                    format!($fmt, $($arg)*)
                )
            )
        }
    };
    ($log: expr, $fmt: expr) => {
        log_minor!($log, "{}", $fmt);
    };
}

#[macro_export]
macro_rules! log_major {
    ($log: expr, $fmt: expr, $($arg: tt)*) => {
        if $log.does_log_major() {
            $log.log_major0(
                format!(
                    "Major: {}",
                    format!($fmt, $($arg)*)
                )
            )
        }
    };
    ($log: expr, $fmt: expr) => {
        log_major!($log, "{}", $fmt);
    };
}

#[macro_export]
macro_rules! log_error {
    ($log: expr, $fmt: expr, $($arg: tt)*) => {
        if $log.does_log_major() {
            $log.log_major0(
                format!(
                    "Error: {}",
                    format!($fmt, $($arg)*)
                )
            )
        }
    };
    ($log: expr, $fmt: expr) => {
        log_major!($log, "{}", $fmt);
    };
}
