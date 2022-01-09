use crate::v3::{
    client::Client, config::Config, data_types::Region, workspace::Workspace, xconnection::Xid,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut, Index, IndexMut},
};

#[derive(Debug)]
pub(crate) struct WmState {
    pub config: Config,
    pub clients: Clients,
    pub screens: Screens,
    pub workspaces: Workspaces,
}

#[derive(Debug, Default)]
pub(super) struct Clients {
    inner: HashMap<Xid, Client>,
    pub focused_id: Option<Xid>,
}

impl Deref for Clients {
    type Target = HashMap<Xid, Client>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Clients {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Clients {
    pub fn remove(&mut self, id: Xid) -> Option<Client> {
        if self.focused_id == Some(id) {
            self.focused_id = None;
        }

        self.inner.remove(&id)
    }

    pub fn focused_client(&self) -> Option<&Client> {
        self.focused_id.and_then(|id| self.inner.get(&id))
    }

    pub fn focused_client_mut(&mut self) -> Option<&mut Client> {
        self.focused_id.and_then(move |id| self.inner.get_mut(&id))
    }
}

#[derive(Debug, Default)]
pub struct Screens {
    pub focused: usize,
    pub workspaces: Vec<usize>,
    pub(crate) inner: Vec<Region>,
}

impl Screens {
    pub fn indexed_screen_for_workspace(&self, wix: usize) -> Option<(usize, Region)> {
        self.workspaces
            .iter()
            .position(|&i| i == wix)
            .map(|i| (i, self.inner[i]))
    }

    pub fn effective_region(&self, ix: usize, bar_height: u32, top_bar: bool) -> Region {
        let (x, y, w, h) = self.inner[ix].values();
        if top_bar {
            Region::new(x, y + bar_height, w, h - bar_height)
        } else {
            Region::new(x, y, w, h - bar_height)
        }
    }
}

impl Deref for Screens {
    type Target = Vec<Region>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Screens {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Index<usize> for Screens {
    type Output = Region;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for Screens {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

/// The current workspace state of Penrose
#[derive(Debug, Default)]
pub(crate) struct Workspaces {
    pub inner: Vec<Workspace>,
    pub focused: usize,
    pub prev: usize,
}

impl Deref for Workspaces {
    type Target = Vec<Workspace>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Workspaces {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Index<usize> for Workspaces {
    type Output = Workspace;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for Workspaces {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}
