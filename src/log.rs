//! # Logging facilities
//!
//! Simple usage: create an instance of either the [`LogNothing`], [`LogFaults`],
//! [`LogMajorsAndFaults`], or [`LogEverything`] structs, then apply one of the
//! [`log_fault`], [`log_major`], or [`log_minor`] macros.
//!
//! For example:
//!
//! ```
//! let log = log::LogEverything(std::io::stdout());
//! log_minor(log, "Something minor happened");
//! log_major(log, "Something major happened");
//! log_fault(log, "A tragic error happened");
//! ```
//! 
//! In increasing order of priority, there are minor, major, and fault log entires.
//! The [`Log`] trait accepts all three of these, but is designed so that
//! types may not record all three, and so that `rustc` can optimize away all
//! calls when logs are not actually recorded. Fault log entries are intended to
//! log faulty Master System code; for instance, attempts to write to ROM.
//!
//! [`Log`]: trait.Log.html
//! [`LogNothing`]: struct.LogNothing.html
//! [`LogFaults`]: struct.LogFaults.html
//! [`LogMajorsAndFaults`]: struct.LogMajorsAndFaults.html
//! [`LogEverything`]: struct.LogEverything.html
//! [`log_fault`]: ../macro.log_error.html
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
    fn log_fault0(&mut self, s: String);

    /// Will this `Log` actually record minor log entries? If constantly `false`,
    /// `rustc` can optimize away calls to the [`log_minor`] macro.
    /// [`log_minor`]: ../macro.log_minor.html
    fn does_log_minor(&self) -> bool;

    /// See [`does_log_minor`].
    /// [`does_log_minor`]: #ty_method.does_log-minor
    fn does_log_major(&self) -> bool;

    /// See [`does_log_minor`].
    /// [`does_log_minor`]: #ty_method.does_log-minor
    fn does_log_fault(&self) -> bool;

    /// Returns `Ok` if no fault has been recorded; else returns a string
    /// indicating the first fault recorded.
    fn check_fault(&self) -> Option<String>;
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

struct LogGeneral<LogMinor: Bool, LogMajor: Bool, LogFault: Bool, W: Write> {
    write: W,
    fault: Option<String>,
    pd1: std::marker::PhantomData<LogMinor>,
    pd2: std::marker::PhantomData<LogMajor>,
    pd3: std::marker::PhantomData<LogFault>,
}

impl<LogMinor: Bool, LogMajor: Bool, LogFault: Bool, W: Write> Log for
  LogGeneral<LogMinor, LogMajor, LogFault, W> {
    fn log_minor0(&mut self, s: String) {
        self.write.write_all(s.as_bytes());
        self.write.write_all(b"\n");
    }
    fn log_major0(&mut self, s: String) {
        self.write.write_all(s.as_bytes());
        self.write.write_all(b"\n");
    }
    fn log_fault0(&mut self, s: String) {
        self.write.write_all(s.as_bytes());
        self.write.write_all(b"\n");
        if self.fault.is_none() {
            self.fault = Some(s);
        }
    }
    fn does_log_minor(&self) -> bool { LogMinor::get() }
    fn does_log_major(&self) -> bool { LogMajor::get() }
    fn does_log_fault(&self) -> bool { LogFault::get() }
    fn check_fault(&self) -> Option<String> {
        self.fault.clone()
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
                fault: None,
                pd1: Default::default(),
                pd2: Default::default(),
                pd3: Default::default(),
            }
        )
    }
}

pub struct LogFaults<W: Write>(LogGeneral<False, False, True, W>);

pub struct LogMajorsAndFaults<W: Write>(LogGeneral<False, True, True, W>);

pub struct LogEverything<W: Write>(LogGeneral<True, True, True, W>);

macro_rules! impl_new {
    ($type_name: ident) => {
        impl<W: Write> $type_name<W> {
            pub fn new(write: W) -> $type_name<W> {
                $type_name(
                    LogGeneral {
                        write: write,
                        fault: None,
                        pd1: Default::default(),
                        pd2: Default::default(),
                        pd3: Default::default(),
                    }
                )
            }
        }
    }
}

impl_new!{LogFaults}

impl_new!{LogMajorsAndFaults}

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
            fn log_fault0(&mut self, s: String) {
                self.0.log_fault0(s)
            }
            fn does_log_minor(&self) -> bool {
                self.0.does_log_minor()
            }
            fn does_log_major(&self) -> bool {
                self.0.does_log_major()
            }
            fn does_log_fault(&self) -> bool {
                self.0.does_log_fault()
            }
            fn check_fault(&self) -> Option<String> {
                self.0.check_fault()
            }
        }
    }
}

impl_log!{[] LogNothing}

impl_log!{[<W: Write>] LogFaults<W>}

impl_log!{[<W: Write>] LogMajorsAndFaults<W>}

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
macro_rules! log_fault {
    ($log: expr, $fmt: expr, $($arg: tt)*) => {
        if $log.does_log_major() {
            $log.log_major0(
                format!(
                    "Fault: {}",
                    format!($fmt, $($arg)*)
                )
            )
        }
    };
    ($log: expr, $fmt: expr) => {
        log_major!($log, "{}", $fmt);
    };
}
