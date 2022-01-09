use crate::v3::xconnection::Xid;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum HookTrigger {
    Startup,
    NewClient(Xid),
    RemoveClient(Xid),
    ClientAddedToWorkspace(Xid, usize),
    ClientNameUpdated(Xid, String, bool),
    LayoutApplied(usize, usize),
    LayoutChange(usize),
    WorkspaceChange(usize, usize),
    WorkspacesUpdated(Vec<String>, usize),
    ScreenChange,
    ScreenUpdated,
    RanderNotify,
    FocusChange(u32),
    EventHandled,
}
