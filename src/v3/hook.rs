//! User defined hooks

// TODO: port over and update the documentation on Hooks once the API is finalised

use crate::v3::{data_types::Region, handle::WmHandle, xconnection::Xid, Result};
use std::cell::Cell;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum HookTrigger {
    Startup,
    NewClient { id: Xid },
    RemoveClient { id: Xid },
    ClientAddedToWorkspace { id: Xid, ws: usize },
    ClientNameUpdated { id: Xid, name: String, root: bool },
    LayoutApplied { ws: usize, screen: usize },
    LayoutChange { ws: usize },
    WorkspaceChange { prev: usize, new: usize },
    WorkspacesUpdated { names: Vec<String>, active: usize },
    ScreenChange { screen: usize },
    ScreenUpdated { rs: Vec<Region> },
    RanderNotify,
    FocusChange { id: u32 },
    EventHandled,
}

/// Utility type for defining hooks in your penrose configuration.
pub type Hooks = Vec<Box<dyn Hook>>;

pub(crate) struct HookRunner {
    inner: Cell<Vec<Box<dyn Hook>>>,
    h: WmHandle,
}

impl HookRunner {
    #[tracing::instrument(level = "trace", skip(self))]
    fn run(&self, t: HookTrigger) -> Result<()> {
        use HookTrigger::*;

        // Relies on all hooks taking WmHandle as the first arg.
        macro_rules! run_hooks {
            ($_self:expr, $method:ident, $($arg:expr),*) => {
                {
                    debug!(target: "hooks", "Running {} hooks", stringify!($method));
                    let mut hooks = $_self.inner.replace(vec![]);
                    let res = hooks.iter_mut().try_for_each(|h| h.$method($_self.h.clone(), $($arg),*));
                    $_self.inner.replace(hooks);
                    res
                }
            };
        }

        match t {
            Startup => run_hooks!(self, startup,),
            NewClient { id } => run_hooks!(self, new_client, id),
            RemoveClient { id } => run_hooks!(self, remove_client, id),
            ClientAddedToWorkspace { id, ws } => {
                run_hooks!(self, client_added_to_workspace, id, ws)
            }
            ClientNameUpdated { id, name, root } => {
                run_hooks!(self, client_name_updated, id, &name, root)
            }
            LayoutApplied { ws, screen } => run_hooks!(self, layout_applied, ws, screen),
            LayoutChange { ws } => run_hooks!(self, layout_change, ws),
            WorkspaceChange { prev, new } => run_hooks!(self, workspace_change, prev, new),
            WorkspacesUpdated { names, active } => {
                run_hooks!(self, workspaces_updated, str_slice!(names), active)
            }
            ScreenChange { screen } => run_hooks!(self, screen_change, screen),
            ScreenUpdated { rs } => run_hooks!(self, screens_updated, &rs),
            RanderNotify => run_hooks!(self, randr_notify,),
            FocusChange { id } => run_hooks!(self, focus_change, id),
            EventHandled => run_hooks!(self, event_handled,),
        }
    }
}

pub trait Hook {
    #[allow(unused_variables)]
    fn startup(&mut self, h: WmHandle) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn new_client(&mut self, h: WmHandle, id: Xid) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn remove_client(&mut self, h: WmHandle, id: Xid) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn client_added_to_workspace(&mut self, h: WmHandle, id: Xid, wix: usize) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn client_name_updated(
        &mut self,
        h: WmHandle,
        id: Xid,
        name: &str,
        is_root: bool,
    ) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn layout_applied(
        &mut self,
        h: WmHandle,
        workspace_index: usize,
        screen_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn layout_change(&mut self, h: WmHandle, workspace_index: usize) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn workspace_change(
        &mut self,
        h: WmHandle,
        previous_workspace: usize,
        new_workspace: usize,
    ) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn workspaces_updated(&mut self, h: WmHandle, names: &[&str], active: usize) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn screen_change(&mut self, h: WmHandle, screen_index: usize) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn screens_updated(&mut self, h: WmHandle, dimensions: &[Region]) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn randr_notify(&mut self, h: WmHandle) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn focus_change(&mut self, h: WmHandle, id: Xid) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn event_handled(&mut self, h: WmHandle) -> Result<()> {
        Ok(())
    }
}
