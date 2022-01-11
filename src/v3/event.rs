//! Central traits for writing Window Manager events and event handlers
//!
//! Penrose uses a webserver like design of request handlers, each of which responds to a single
//! concrete Event type. Requests are procressed sequentially in the main program loop (not
//! concurrently) to ensure ordering with the events from the X server.
use crate::v3::{
    bindings::{KeyBindings, MouseBindings},
    error::ErrorHandler,
    manager::WindowManager,
    state::State,
    xconnection::XConn,
    Result,
};
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};

pub trait Event: Debug {
    /// Is this event only accepted from the X server itself.
    const X_ONLY: bool;
}

#[cfg(feature = "serde")]
/// An Event that can be serialized and deserialized
pub trait SerializableEvent: Event + Serialize + DeserializeOwned {}

pub trait Handler {
    type Event: Event;

    fn handle(&self, e: Self::Event, s: &mut State) -> Result<()>;
}

pub fn event_loop<X>(
    wm: WindowManager<X>,
    mut key_bindings: KeyBindings,
    mut mouse_bindings: MouseBindings,
    error_handler: ErrorHandler,
) -> Result<()>
where
    X: XConn,
{
    Ok(())
}
