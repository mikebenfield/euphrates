use std::fmt::{self, Debug, Display};

use failure::{Backtrace, Context, Fail};

pub trait Kind: Display + Send + Sync + Debug + Clone + PartialEq + 'static {}

#[derive(Debug)]
pub struct Error<K: Kind>(Context<K>);

impl<K: Kind> Display for Error<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<K: Kind> Fail for Error<K> {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }
}

impl<K: Kind> Error<K> {
    pub fn kind(&self) -> K {
        self.0.get_context().clone()
    }
}

impl<K: Kind> From<K> for Error<K> {
    fn from(kind: K) -> Error<K> {
        Error(Context::new(kind))
    }
}

impl<K: Kind> From<Context<K>> for Error<K> {
    fn from(context: Context<K>) -> Error<K> {
        Error(context)
    }
}

impl Kind for String {}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CommonKind {
    /// Some error has corrupted the object in question and it will no longer produce
    /// correct results.
    Dead(String),
    /// An error has occurred, but the object may still be used.
    Live(String),
}

impl Kind for CommonKind {}

impl Display for CommonKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &CommonKind::Dead(ref s) => format!("fatal error: {}", s),
            &CommonKind::Live(ref s) => format!("error: {}", s),
        };

        Display::fmt(&s, f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SimpleKind(pub String);

impl Kind for SimpleKind {}

impl Display for SimpleKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}
