//! Shared error and result types

type Xid = u32;

/// Top level penrose Result type
pub type Result<T> = std::result::Result<T, Error>;

/// A function that can be registered to handle errors that occur during WindowManager operation
pub type ErrorHandler = Box<dyn FnMut(Error)>;

/// Top level error types returned by Penrose
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[doc(hidden)]
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),

    /// An [IO Error][std::io::Error] was encountered
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Wm(Normal)Hints received from the X server were invalid
    #[error("Invalid window hints property: {0}")]
    InvalidHints(String),

    /// No elements match the given selector
    #[error("No elements match the given selector")]
    NoMatchingElement,

    /// A generic error type for use in user code when needing to construct
    /// a simple [PenroseError].
    #[error("Unhandled error: {0}")]
    Raw(String),

    /// An attempt to spawn an external process failed
    #[error("unable to get stdout handle for child process: {0}")]
    SpawnProc(String),

    /// Parsing an [Atom][core::xconnection::Atom] from a str failed.
    ///
    /// This happens when the atom name being requested is not a known atom.
    #[error(transparent)]
    Strum(#[from] strum::ParseError),

    /// An attempt was made to reference a client that is not known to penrose
    #[error("{0} is not a known client id")]
    UnknownClient(Xid),

    /// A user specified key binding contained an invalid modifier key
    #[error("Unknown modifier key: {0}")]
    UnknownModifier(String),

    /// A conversion to utf-8 failed
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Something went wrong when communicating with the X server
    #[error(transparent)]
    X(#[from] crate::v3::xconnection::XError),
}
