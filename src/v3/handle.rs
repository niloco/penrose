//! A handle to the WindowManager for submitting requests
use crate::v3::{rpc::Rpc, Result};
use crossbeam_channel::{bounded, Sender};

#[derive(Debug, Clone)]
pub struct WmHandle {
    tx: Sender<Rpc>,
}

macro_rules! rpc_call {
    ($self:expr, $kind:ident => $($param:ident),*) => ({
        let (tx, rx) = bounded(1);
        let r = Rpc::$kind { tx: Some(tx), $($param),* };
        $self.tx.send(r).unwrap();
        rx.recv().unwrap() // TODO: Add variant to Error
    })
}

impl WmHandle {
    pub(crate) fn new(tx: Sender<Rpc>) -> Self {
        Self { tx }
    }

    pub fn add_client_to_workspace(&self, id: u32, ws: usize) -> Result<()> {
        rpc_call!(self, ClientToWorkspace => id, ws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_add_client_to_ws() {
        let (tx, rx) = bounded(1);
        let h = WmHandle::new(tx);

        let handle = thread::spawn(move || {
            let res = h.add_client_to_workspace(1, 2);
            assert!(res.is_ok());
        });

        match rx.recv().unwrap() {
            Rpc::ClientToWorkspace {
                id,
                ws,
                tx: Some(tx),
            } => {
                assert_eq!((id, ws), (1, 2));
                tx.send(Ok(())).unwrap();
            }

            e => panic!("expected AddClientToWorkspace, got {:?}", e),
        }

        handle.join().unwrap();
    }
}
