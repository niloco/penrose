//! RPC calls to the main window manager event loop.
//!
//! The calls are made from:
//!   - the X server event poll loop
//!   - user Hooks
//!   - user code run as part of a key or mouse binding
use crate::v3::{
    bindings::{KeyCode, MouseEvent},
    data_types::{Change, Point, Region},
    error::Result,
    hook::HookTrigger,
    ring::InsertPoint,
    state::Screens,
    workspace::Workspace,
};
use crossbeam_channel::Sender;

type TxRes = Option<Sender<Result<()>>>;
type TxO<T> = Sender<Option<T>>;
type TxV<T> = Sender<Vec<T>>;
type Tx<T> = Sender<T>;

#[derive(Debug)]
pub enum Rpc {
    Exit,
    RunHook { h: HookTrigger },
    RunKeyBinding { k: KeyCode },
    RunMouseBinding { e: MouseEvent },
    SetInsertPoint { ip: InsertPoint },
    SetRootWindowName { s: String, tx: TxRes },

    // Client
    ClientToScreen { id: u32, s: usize, tx: TxRes },
    ClientToWorkspace { id: u32, ws: usize, tx: TxRes },
    FocusClient { id: u32, tx: TxRes },
    HideClient { id: u32, tx: TxRes },
    KillClient { id: u32, tx: TxRes },
    PositionClient { id: u32, r: Region, tx: TxRes },
    SetActiveClient { id: u32, tx: TxRes },
    ShowClient { id: u32, tx: TxRes },
    ToggleFullScreen { id: u32, fs: bool, tx: TxRes },
    ToggleWorkspace,
    WorkspaceClients { ws: usize, tx: TxV<Workspace> },

    // Workspace
    Workspaces { tx: TxV<Workspace> },
    AddWorkspace { ix: usize, ws: Workspace },
    FocusWorkspace { ws: usize },
    RemoveWorkspace { ix: usize, tx: TxO<Workspace> },
    ApplyLayout { ws: Option<usize>, tx: TxRes },
    RenameWorkspace { ws: usize, s: String, tx: TxRes },
    UpdateMaxMain { c: Change },
    UpdateRatio { c: Change },

    // Screen
    Screens { tx: Tx<Screens> },
    DetectScreens,
    FocusScreen { s: usize, tx: TxRes },
    SetScreenFromPoint { p: Option<Point> },

    // X Server Events
    XMapRequest { id: u32 },
    XUnmapNotify { id: u32 },
    UpdateXClients,
    UpdateXWorkspaces,
}
