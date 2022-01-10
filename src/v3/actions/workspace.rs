use crate::v3::{
    hook::HookTrigger,
    state::{Screens, Workspaces},
    xconnection::XConn,
    Result,
};

pub fn focus_workspace<X>(index: usize, s: &mut Screens, w: &mut Workspaces, x: &X) -> Result<()>
where
    X: XConn,
{
    if w.focused == index {
        return Ok(());
    }

    let active = s.workspaces[s.focused];
    w.prev = active;

    for i in 0..s.len() {
        if s.workspaces[i] == index {
            // The workspace we want is currently displayed on another screen so
            // pull the target workspace to the focused screen, and place the
            // workspace we had on the screen where the target was
            s.workspaces[i] = active;
            s.workspaces[s.focused] = index;

            // re-apply layouts as screen dimensions may differ
            apply_layout(active)?;
            apply_layout(index)?;

            x.set_current_workspace(index)?;

            if let Some(id) = w[index].focused_client() {
                update_focus(id)?;
            };

            w.focused = index;

            run_hook(HookTrigger::WorkspaceChange {
                prev: active,
                new: index,
            });

            return Ok(());
        }
    }

    // target not currently displayed so unmap what we currently have
    // displayed and replace it with the target workspace
    for id in w[active].client_ids().iter() {
        unmap_if_needed(*id, x)?;
    }

    for id in w[index].client_ids().iter() {
        map_if_needed(*id, x)?;
    }

    s.workspaces[s.focused] = index;
    apply_layout(index)?;
    x.set_current_workspace(index)?;

    if let Some(id) = w[index].focused_client() {
        update_focus(id)?;
    };

    w.focused = index;
    run_hook(HookTrigger::WorkspaceChange {
        prev: active,
        new: index,
    });

    Ok(())
}

// TODO: these all need moving to the correct files

fn apply_layout(ix: usize) -> Result<()> {
    todo!()
}

fn run_hook(t: HookTrigger) {
    todo!()
}

fn update_focus(id: u32) -> Result<()> {
    todo!()
}

fn map_if_needed<X>(id: u32, x: &X) -> Result<()>
where
    X: XConn,
{
    todo!()
}

fn unmap_if_needed<X>(id: u32, x: &X) -> Result<()>
where
    X: XConn,
{
    todo!()
}
