//! The window manager
use crate::v3::{
    actions,
    bindings::{KeyBindings, MouseBindings},
    config::Config,
    state::WmState,
    xconnection::XConn,
    Error, ErrorHandler, Result,
};
use nix::sys::signal::{signal, SigHandler, Signal};

/// WindowManager is the primary struct / owner of the event loop for penrose.
///
/// It handles most (if not all) of the communication with the underlying [XConn], responding to
/// [XEvent][crate::core::xconnection::XEvent]s emitted by it. User key / mouse bindings are parsed
/// and bound on the call to `grab_keys_and_run` and then triggered when corresponding `XEvent`
/// instances come through in the main event loop.
///
/// # A note on examples
///
/// The examples provided for each of the `WindowManager` methods are written using an example
/// implementation of [XConn] that mocks out calls to the X server. In each case, it is assumed
/// that you have an initialised `WindowManager` struct as demonstrated in the full examples for
/// `new` and `init`.
///
/// For full examples of how to configure the `WindowManager`, please see the [examples][1]
/// directory in the Penrose repo.
///
/// [1]: https://github.com/sminez/penrose/tree/develop/examples
#[derive(Debug)]
pub struct WindowManager<X: XConn> {
    c: Config,
    x: X,
    s: WmState,
    running: bool,
}

impl<X: XConn> WindowManager<X> {
    /// This is the main event loop for the [WindowManager].
    ///
    /// The `XConn` [wait_for_event][1] method is called to fetch the next event from the X server,
    /// after which it is processed into a set of internal EventActions which are then processed by
    /// the [WindowManager] to update state and perform actions. This method is an infinite loop
    /// until the [exit][2] method is called, which triggers the `XConn` [cleanup][3] before
    /// exiting the loop. You can provide any additional teardown logic you need your main.rs after
    /// the call to `grab_keys_and_run` and all internal state will still be accessible, though
    /// methods requiring the use of the [XConn] will fail.
    ///
    /// [1]: crate::core::xconnection::XEventHandler::wait_for_event
    /// [2]: WindowManager::exit
    /// [3]: crate::core::xconnection::XConn::cleanup
    pub fn run(
        self,
        mut key_bindings: KeyBindings,
        mut mouse_bindings: MouseBindings,
        error_handler: ErrorHandler,
    ) -> Result<()> {
        // ignore SIGCHILD and allow child / inherited processes to be inherited by pid1
        trace!("registering SIGCHILD signal handler");
        if let Err(e) = unsafe { signal(Signal::SIGCHLD, SigHandler::SigIgn) } {
            panic!("unable to set signal handler: {}", e);
        }

        trace!("Initialising XConn");
        self.x.init()?;

        trace!("Attempting initial screen detection");
        actions::screen::detect_screens(&mut self.s.screens, &self.x, self.c.workspaces.len())?;

        trace!("Setting EWMH properties");
        self.x.set_wm_properties(&self.c.workspaces)?;

        trace!("Forcing cursor to first screen");
        self.x.warp_cursor(None, &self.s.screens[0])?;

        trace!("grabbing key and mouse bindings");
        self.x.grab_keys(&key_bindings, &mouse_bindings)?;

        trace!("forcing focus to first workspace");
        self.focus_workspace(&Selector::Index(0))?;

        self.run_hook(HookName::Startup);
        self.running = true;

        trace!("entering main event loop");
        while self.running {
            match self.x.wait_for_event() {
                Ok(event) => {
                    trace!(details = ?event, "event details");

                    let actions = process_next_event(event, &self.state, &self.x);
                    for action in actions {
                        if let Err(e) = self.handle_event_action(
                            action,
                            Some(&mut key_bindings),
                            Some(&mut mouse_bindings),
                        ) {
                            error_handler(e);
                        }
                    }

                    self.run_hook(HookName::EventHandled);
                    self.x.flush();
                }

                Err(e) => error_handler(Error::X(e)),
            }
        }

        Ok(())
    }
}
