//! Setting up and responding to user defined key/mouse bindings
use std::collections::HashMap;

mod keyboard;
mod mouse;

pub use keyboard::*;
pub use mouse::*;

pub(crate) type CodeMap = HashMap<String, u8>;
