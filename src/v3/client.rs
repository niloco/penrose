//! Metadata around X clients
use crate::v3::xconnection::{Atom, Prop, WmHints, WmNormalHints, XClientProperties, Xid};

/// Meta-data around a client window that we are handling.
///
/// Primarily state flags and information used when determining which clients
/// to show for a given monitor and how they are tiled.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Client {
    /// The X Client ID of this client
    pub id: Xid,
    // state flags
    pub(crate) accepts_focus: bool,
    pub(crate) floating: bool,
    pub(crate) fullscreen: bool,
    pub(crate) mapped: bool,
    pub(crate) urgent: bool,
    pub(crate) wm_managed: bool,
    pub(crate) wm_name: String,
    pub(crate) wm_class: Vec<String>, // should always be two elements but that's not enforced?
    pub(crate) wm_type: Vec<String>,  // Can't use Atom as it could be something arbitrary
    pub(crate) wm_protocols: Vec<String>, // Can't use Atom as it could be something arbitrary
    pub(crate) wm_hints: Option<WmHints>,
    pub(crate) wm_normal_hints: Option<WmNormalHints>,
}

impl Client {
    pub(crate) fn new<X>(id: Xid, fcs: &[&str], x: X) -> Self
    where
        X: XClientProperties,
    {
        let floating = x.client_should_float(id, fcs);
        let accepts_focus = x.client_accepts_focus(id);
        let wm_name = x.client_name(id).unwrap_or("unknown".into());

        let wm_class = match x.get_prop(id, Atom::WmClass.as_ref()) {
            Ok(Prop::UTF8String(strs)) => strs,
            _ => vec![],
        };
        let wm_type = match x.get_prop(id, Atom::NetWmWindowType.as_ref()) {
            Ok(Prop::Atom(atoms)) => atoms,
            _ => vec![Atom::NetWindowTypeNormal.as_ref().to_string()],
        };
        let wm_hints = match x.get_prop(id, Atom::WmHints.as_ref()) {
            Ok(Prop::WmHints(hints)) => Some(hints),
            _ => None,
        };
        let wm_normal_hints = match x.get_prop(id, Atom::WmNormalHints.as_ref()) {
            Ok(Prop::WmNormalHints(hints)) => Some(hints),
            _ => None,
        };
        let wm_protocols = match x.get_prop(id, Atom::WmProtocols.as_ref()) {
            Ok(Prop::Atom(protocols)) => protocols,
            _ => vec![],
        };

        Self {
            id,
            wm_name,
            wm_class,
            wm_type,
            wm_protocols,
            wm_hints,
            wm_normal_hints,
            floating,
            accepts_focus,
            fullscreen: false,
            mapped: false,
            urgent: false,
            wm_managed: true,
        }
    }
}
