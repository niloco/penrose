//! A set of clients and accompanying layout logic for display on a single screen
//!
//! The [Workspace] struct is Penrose' control structure for what should be displayed on a single
//! screen at any one point. Each individual [Client] is owned centrally by the [WindowManager][1]
//! but can be obtained via its ID which is tracked in the `Workspace`. [Layouts][2] are managed per
//! workspace, allowing you to specialise layout behaviour for individual workspaces if desired.
//!
//! [1]: crate::core::manager::WindowManager
//! [2]: crate::core::layout::Layout
use crate::v3::{
    data_types::Change,
    layout::{Layout, LayoutConf},
    ring::{Direction, InsertPoint, Ring},
    xconnection::Xid,
    Error, Result,
};

mod arrange_actions;

pub use arrange_actions::*;

/// A Workspace represents a named set of clients that are tiled according
/// to a specific layout. Layout properties are tracked per workspace and
/// clients are referenced by ID. Workspaces are independent of monitors and
/// can be moved between monitors freely, bringing their clients with them.
///
/// The parent WindowManager struct tracks which client is focused from the
/// point of view of the X server by checking focus at the Workspace level
/// whenever a new Workspace becomes active.
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    /// The internal name for this workspace
    pub name: String,
    pub(crate) layouts: Ring<Layout>,
    pub(crate) clients: Ring<Xid>,
}

impl Workspace {
    /// Construct a new Workspace with the given name and choice of [layouts][1]
    ///
    /// [1]: crate::core::layout::Layout
    pub fn new(name: impl Into<String>, layouts: Vec<Layout>) -> Self {
        if layouts.is_empty() {
            panic!("{}: require at least one layout function", name.into());
        }

        Self {
            name: name.into(),
            layouts: layouts.into(),
            clients: Ring::new(),
        }
    }

    /// The number of clients currently on this workspace
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Is this Workspace currently empty?
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Iterate over the clients on this workspace in position order
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, Xid> {
        self.clients.iter()
    }

    /// The ordered list of [Client] IDs currently contained in this workspace
    ///
    /// # Example
    ///
    /// ```
    /// # use penrose::__test_helpers::*;
    /// # fn example(mut workspace: Workspace) -> Result<()> {
    /// assert_eq!(workspace.client_ids(), vec![0, 1, 2, 3, 4]);
    /// # Ok(())
    /// # }
    /// # example(test_workspace("example", 5)).unwrap();
    /// ```
    pub fn client_ids(&self) -> &[Xid] {
        self.clients.elements()
    }

    pub(crate) fn focused_client(&self) -> Option<Xid> {
        self.clients.focused_element().copied()
    }

    pub(crate) fn add_client(&mut self, id: Xid, ip: InsertPoint) -> Result<()> {
        if self.clients.contains(&id) {
            return Err(Error::Raw(format!("{} is already in this workspace", id)));
        }

        self.clients.insert(id, ip);

        Ok(())
    }

    pub(crate) fn remove_client(&mut self, id: Xid) -> Option<Xid> {
        self.clients
            .position(|&c| c == id)
            .and_then(|index| self.clients.remove(index))
    }

    /// Set the active layout by symbol name if it is available. Returns a reference to active
    /// layout if it was able to be set.
    ///
    /// # Example
    ///
    /// ```
    /// # use penrose::__test_helpers::*;
    /// # fn example(mut workspace: Workspace) -> Result<()> {
    /// assert_eq!(workspace.layout_symbol(), "first");
    ///
    /// assert!(workspace.try_set_layout("second").is_some());
    /// assert_eq!(workspace.layout_symbol(), "second");
    ///
    /// assert!(workspace.try_set_layout("invalid").is_none());
    /// assert_eq!(workspace.layout_symbol(), "second");
    /// # Ok(())
    /// # }
    /// # example(test_workspace("example", 2)).unwrap();
    /// ```
    pub fn try_set_layout(&mut self, symbol: &str) -> Result<&Layout> {
        self.layouts
            .focus(|l| l.symbol == symbol)
            .map(|(_, index)| &self.layouts[index])
    }

    /// Cycle through the available layouts on this workspace
    ///
    /// # Example
    ///
    /// ```
    /// # use penrose::__test_helpers::*;
    /// # fn example(mut workspace: Workspace) -> Result<()> {
    /// assert_eq!(workspace.layout_symbol(), "first");
    /// assert_eq!(workspace.cycle_layout(Forward), "second");
    /// assert_eq!(workspace.cycle_layout(Forward), "first");
    /// # Ok(())
    /// # }
    /// # example(test_workspace("example", 2)).unwrap();
    /// ```
    pub fn cycle_layout(&mut self, direction: Direction) -> &str {
        self.layouts.rotate(direction);
        self.layout_symbol()
    }

    /// The symbol of the currently used layout (passed on creation)
    ///
    /// # Example
    ///
    /// ```
    /// # use penrose::__test_helpers::*;
    /// # fn example(mut workspace: Workspace) -> Result<()> {
    /// assert_eq!(workspace.layout_symbol(), "first");
    /// # Ok(())
    /// # }
    /// # example(test_workspace("example", 2)).unwrap();
    /// ```
    pub fn layout_symbol(&self) -> &str {
        &self.layouts.focused_element_unchecked().symbol
    }

    pub(crate) fn current_layout(&self) -> &Layout {
        self.layouts.focused_element_unchecked()
    }

    pub(crate) fn current_layout_config(&self) -> LayoutConf {
        self.layouts.focused_element_unchecked().conf
    }

    /// Cycle focus through the clients on this workspace, returning the previous and new focused
    /// client ids.
    ///
    /// # Example
    ///
    /// ```
    /// # use penrose::__test_helpers::*;
    /// # fn example(mut workspace: Workspace) -> Result<()> {
    /// assert_eq!(workspace.client_ids(), vec![0, 1, 2]);
    /// assert_eq!(workspace.focused_client(), Some(0));
    /// assert_eq!(workspace.cycle_client(Forward), Some((0, 1)));
    /// assert_eq!(workspace.cycle_client(Forward), Some((1, 2)));
    /// assert_eq!(workspace.cycle_client(Forward), Some((2, 0)));
    /// # Ok(())
    /// # }
    /// # example(test_workspace("example", 3)).unwrap();
    /// ```
    pub fn cycle_client(&mut self, direction: Direction) -> Option<(Xid, Xid)> {
        if self.clients.len() < 2 {
            return None; // need at least two clients to cycle
        }

        if !self.current_layout_config().allow_wrapping && self.clients.would_wrap(direction) {
            return None;
        }

        let prev = self.focused_client()?;
        let new = *self.clients.cycle_focus(direction)?;

        Some((prev, new))
    }

    /// Drag the focused client through the stack, retaining focus
    ///
    /// # Example
    ///
    /// ```
    /// # use penrose::__test_helpers::*;
    /// # fn example(mut workspace: Workspace) -> Result<()> {
    /// assert_eq!(workspace.client_ids(), vec![0, 1, 2]);
    /// assert_eq!(workspace.focused_client(), Some(0));
    ///
    /// workspace.drag_client(Forward), Some(0);
    /// assert_eq!(workspace.client_ids(), vec![1, 0, 2]);
    /// assert_eq!(workspace.focused_client(), Some(0));
    /// # Ok(())
    /// # }
    /// # example(test_workspace("example", 3)).unwrap();
    /// ```
    pub fn drag_client(&mut self, direction: Direction) {
        if !self.current_layout_config().allow_wrapping && self.clients.would_wrap(direction) {
            return;
        }

        self.clients.drag_focused(direction);
    }

    /// Rotate the client stack in the given direction
    ///
    /// # Example
    ///
    /// ```
    /// # use penrose::__test_helpers::*;
    /// # fn example(mut workspace: Workspace) -> Result<()> {
    /// assert_eq!(workspace.client_ids(), vec![0, 1, 2, 3]);
    /// assert_eq!(workspace.focused_client(), Some(0));
    ///
    /// workspace.rotate_clients(Forward);
    /// assert_eq!(workspace.client_ids(), vec![3, 0, 1, 2]);
    /// assert_eq!(workspace.focused_client(), Some(3));
    ///
    /// workspace.rotate_clients(Forward);
    /// assert_eq!(workspace.client_ids(), vec![2, 3, 0, 1]);
    /// assert_eq!(workspace.focused_client(), Some(2));
    /// # Ok(())
    /// # }
    /// # example(test_workspace("example", 4)).unwrap();
    /// ```
    pub fn rotate_clients(&mut self, direction: Direction) {
        self.clients.rotate(direction)
    }

    /// Increase or decrease the number of possible clients in the main area of the current Layout
    pub fn update_max_main(&mut self, change: Change) {
        self.layouts
            .focused_element_mut_unchecked()
            .update_max_main(change);
    }

    /// Increase or decrease the size of the main area for the current Layout
    pub fn update_main_ratio(&mut self, change: Change, step: f32) {
        self.layouts
            .focused_element_mut_unchecked()
            .update_main_ratio(change, step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new::{layout::*, ring::Direction::*};
    use test_case::test_case;

    fn test_layouts() -> Vec<Layout> {
        vec![Layout::new("t", LayoutConf::default(), mock_layout, 1, 0.6)]
    }

    #[test_case(vec![], None; "empty")]
    #[test_case(vec![1, 2], Some(1); "populated")]
    fn focused_client(raw: Vec<Xid>, expected: Option<Xid>) {
        let mut ws = Workspace::new("test", test_layouts());
        ws.clients = Ring::from(raw);

        assert_eq!(ws.focused_client(), expected);
    }

    #[test_case(2, Some(2); "present")]
    #[test_case(42, None; "not present")]
    fn remove_client(target: Xid, expected: Option<Xid>) {
        let mut ws = Workspace::new("test", test_layouts());
        ws.clients = Ring::from(vec![1, 2, 3]);

        assert_eq!(ws.remove_client(target), expected);
    }

    #[test]
    fn add_client() {
        let mut ws = Workspace::new("test", test_layouts());
        ws.clients = Ring::from(vec![2, 3]);

        let res = ws.add_client(1, InsertPoint::First);

        assert!(res.is_ok());
        assert_eq!(ws.client_ids(), &[1, 2, 3])
    }

    #[test]
    fn add_client_duplicate_is_error() {
        let mut ws = Workspace::new("test", test_layouts());
        ws.clients = Ring::from(vec![2, 3]);

        let res = ws.add_client(2, InsertPoint::First);

        assert!(res.is_err());
        assert_eq!(ws.client_ids(), &[2, 3])
    }

    #[test_case(Forward, 1, true, &[1, 3, 2]; "forward")]
    #[test_case(Backward, 1, true, &[2, 1, 3]; "backward")]
    #[test_case(Forward, 2, true, &[3, 1, 2]; "forward wrap")]
    #[test_case(Backward, 0, true, &[2, 3, 1]; "backward wrap")]
    #[test_case(Forward, 2, false, &[1, 2, 3]; "forward no wrap")]
    #[test_case(Backward, 0, false, &[1, 2, 3]; "backward no wrap")]
    fn drag_client(d: Direction, focused: usize, allow_wrapping: bool, expected: &[Xid]) {
        let mut conf = LayoutConf::default();
        conf.allow_wrapping = allow_wrapping;
        let layouts = vec![Layout::new("t", conf, mock_layout, 1, 0.6)];

        let mut ws = Workspace::new("test", layouts);
        ws.clients = Ring::from(vec![1, 2, 3]);
        ws.clients.focused = focused;

        ws.drag_client(d);
        assert_eq!(ws.client_ids(), expected);
    }
}
