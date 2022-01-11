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
use std::{fmt::Debug, thread};

pub(crate) fn event_loop<X>(
    wm: WindowManager<X>,
    mut key_bindings: KeyBindings,
    mut mouse_bindings: MouseBindings,
    error_handler: ErrorHandler,
) -> Result<()>
where
    X: XConn + 'static,
{
    let WindowManager { rx, x, c, mut s } = wm;
    s.running = true;

    // Spawn x thread
    let tx = s.tx.clone();
    let mut x_running = true;

    let x_handle = thread::spawn(|| {
        while x_running {
            let x_event = &x.wait_for_event().unwrap();
            println!("{:?}", x_event);
        }
    });

    // Spawn worker pool
    let tx = s.tx.clone();
    let worker_handle = thread::spawn(|| {});

    while s.running {
        match rx.recv() {
            Ok(evt) => {
                if let Err(err) = evt.handle(&mut s) {
                    error_handler(err);
                }
            }

            Err(e) => panic!("{}", e),
        }
    }

    // Clean up threads

    Ok(())
}

pub(crate) trait Event: Debug {
    fn handle(&self, s: &mut State) -> Result<()>;
}

/// Signal shutdown for the main event loop which will then clean up the X and worker pool threads
/// before exiting.
#[derive(Debug)]
pub(crate) struct ShutDown;

impl Event for ShutDown {
    fn handle(&self, s: &mut State) -> Result<()> {
        s.running = false;
        Ok(())
    }
}
