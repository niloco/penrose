use super::super::{
    error::{Error, Result},
    handle::WmHandle,
};
use std::{collections::HashMap, convert::TryFrom};
use strum::EnumIter;

#[cfg(feature = "keysyms")]
use penrose_keysyms::XKeySym;

/// Some action to be run by a user key binding
pub type KeyEventHandler = Box<dyn FnMut(WmHandle) -> Result<()>>;

/// User defined key bindings
pub type KeyBindings = HashMap<KeyCode, KeyEventHandler>;

/// Abstraction layer for working with key presses
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPress {
    /// A raw character key
    Utf8(String),
    /// Return / enter key
    Return,
    /// Escape
    Escape,
    /// Tab
    Tab,
    /// Backspace
    Backspace,
    /// Delete
    Delete,
    /// PageUp
    PageUp,
    /// PageDown
    PageDown,
    /// Up
    Up,
    /// Down
    Down,
    /// Left
    Left,
    /// Right
    Right,
}

#[cfg(feature = "keysyms")]
impl TryFrom<XKeySym> for KeyPress {
    type Error = Error;

    fn try_from(s: XKeySym) -> Result<KeyPress> {
        Ok(match s {
            XKeySym::XK_Return | XKeySym::XK_KP_Enter | XKeySym::XK_ISO_Enter => KeyPress::Return,
            XKeySym::XK_Escape => KeyPress::Escape,
            XKeySym::XK_Tab | XKeySym::XK_ISO_Left_Tab | XKeySym::XK_KP_Tab => KeyPress::Tab,
            XKeySym::XK_BackSpace => KeyPress::Backspace,
            XKeySym::XK_Delete | XKeySym::XK_KP_Delete => KeyPress::Delete,
            XKeySym::XK_Page_Up | XKeySym::XK_KP_Page_Up => KeyPress::PageUp,
            XKeySym::XK_Page_Down | XKeySym::XK_KP_Page_Down => KeyPress::PageDown,
            XKeySym::XK_Up | XKeySym::XK_KP_Up => KeyPress::Up,
            XKeySym::XK_Down | XKeySym::XK_KP_Down => KeyPress::Down,
            XKeySym::XK_Left | XKeySym::XK_KP_Left => KeyPress::Left,
            XKeySym::XK_Right | XKeySym::XK_KP_Right => KeyPress::Right,
            s => KeyPress::Utf8(s.as_utf8_string()?),
        })
    }
}

/// A u16 X key-code bitmask
pub type KeyCodeMask = u16;

/// A u8 X key-code enum value
pub type KeyCodeValue = u8;

/// A key press and held modifiers
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeyCode {
    /// The held modifier mask
    pub mask: KeyCodeMask,
    /// The key code that was held
    pub code: KeyCodeValue,
}

impl KeyCode {
    /// Create a new [KeyCode] from this one that removes the given mask
    pub fn ignoring_modifier(&self, mask: KeyCodeMask) -> KeyCode {
        KeyCode {
            mask: self.mask & !mask,
            code: self.code,
        }
    }
}

/// Known modifier keys for bindings
#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ModifierKey {
    /// Control
    Ctrl,
    /// Alt
    Alt,
    /// Shift
    Shift,
    /// Meta / super / windows
    Meta,
}

impl TryFrom<&str> for ModifierKey {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        match s {
            "C" => Ok(Self::Ctrl),
            "A" => Ok(Self::Alt),
            "S" => Ok(Self::Shift),
            "M" => Ok(Self::Meta),
            _ => Err(Error::UnknownModifier(s.into())),
        }
    }
}
