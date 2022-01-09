use crate::v3::{data_types::Region, state::Clients, workspace::Workspace, xconnection::Xid};
use tracing::debug;

/// Output of a Layout function: the new position a window should take
pub type ResizeAction = (Xid, Option<Region>);

#[derive(Debug, Default)]
pub(crate) struct ArrangeActions {
    pub(crate) actions: Vec<ResizeAction>,
    pub(crate) floating: Vec<Xid>,
}

impl ArrangeActions {
    pub(crate) fn new(ws: &Workspace, r: &Region, cs: &Clients) -> Self {
        if ws.is_empty() {
            return Self::default();
        }

        let (floating, tiled) = cs.partitioned_clients_for_workspace(ws);
        let layout = ws.current_layout();

        debug!(
            workspace = ?ws.name,
            layout = ?layout.symbol,
            region = ?r,
            n_tiled = tiled.len(),
            n_floating = floating.len(),
            "applying layout function",
        );

        Self {
            actions: layout.arrange(&tiled, ws.focused_client(), r),
            floating: floating.iter().map(|&c| c.id).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v3::{client::Client, layout::*, ring::Ring};

    fn test_layouts() -> Vec<Layout> {
        vec![Layout::new("t", LayoutConf::default(), mock_layout, 1, 0.6)]
    }

    #[test]
    fn arrange_gives_one_action_per_client() {
        let mut ws = Workspace::new("test", test_layouts());
        ws.clients = Ring::from(vec![1, 2, 3]);

        let mut clients = Clients::default();
        for c in [Client::stub(1), Client::stub(2), Client::stub(3)] {
            clients.insert(c.id, c);
        }

        let ArrangeActions { actions, floating } =
            ArrangeActions::new(&ws, &Region::new(0, 0, 2000, 1000), &clients);

        assert_eq!(actions.len(), 3);
        assert_eq!(floating.len(), 0);
    }
}
