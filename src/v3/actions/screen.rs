use crate::v3::{hook::HookTrigger, rpc::Rpc, state::Screens, xconnection::XState, Result};

#[tracing::instrument(level = "trace", err, skip(s))]
pub fn detect_screens<S>(screens: &mut Screens, s: &S, n_ws: usize) -> Result<Vec<Rpc>>
where
    S: XState,
{
    let detected = s.current_screens()?;

    if screens.inner != detected {
        for r in detected.iter() {
            info!(w = r.w, h = r.h, "screen detected");
        }
        screens.inner = detected;

        let n = detected.len();
        let m = screens.workspaces.len();

        if n < m {
            screens.workspaces.resize(n, 0);
        } else if n > m {
            screens.workspaces.append(
                &mut (0..n_ws)
                    .filter(|w| !screens.workspaces.contains(w))
                    .take(m - n)
                    .collect(),
            );
        }

        Ok(vec![
            Rpc::ApplyLayout { ws: None, tx: None },
            Rpc::RunHook {
                h: HookTrigger::ScreenUpdated {
                    rs: screens.inner.clone(),
                },
            },
        ])
    } else {
        Ok(vec![])
    }
}
