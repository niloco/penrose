//! V0.3 rewrite of the internals of Penrose

pub mod actions;
pub mod bindings;
pub mod client;
pub mod config;
pub mod data_types;
pub mod error;
pub mod event;
pub mod handle;
pub mod hook;
pub mod layout;
pub mod manager;
pub mod ring;
pub mod rpc;
pub mod state;
pub mod worker;
pub mod workspace;
pub mod xconnection;

pub use error::{Error, ErrorHandler, Result};
