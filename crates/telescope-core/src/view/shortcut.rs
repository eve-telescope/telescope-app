//! Global shortcut model, stored in settings as a Tauri accelerator string
//! such as `CommandOrControl+Shift+V`.

use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub cmd_or_ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifiers {
    pub fn any(self) -> bool {
        self.cmd_or_ctrl || self.alt || self.shift
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    /// Non-modifier key; single characters are stored uppercase.
    pub key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutValidation {
    Ok,
    /// Missing a modifier or a key.
    Invalid,
    /// Valid, but shadows a common system or app shortcut.
    Conflict(&'static str),
}

impl ShortcutValidation {
    pub fn is_valid(self) -> bool {
        !matches!(self, Self::Invalid)
    }

    pub fn message(self) -> Option<String> {
        match self {
            Self::Ok => None,
            Self::Invalid => Some("Must have a modifier key".to_string()),
            Self::Conflict(action) => Some(format!("Usually \"{action}\" — will be overridden")),
        }
    }
}

const CONFLICTS: [(&str, &str); 22] = [
    ("CommandOrControl+C", "Copy"),
    ("CommandOrControl+V", "Paste"),
    ("CommandOrControl+X", "Cut"),
    ("CommandOrControl+Z", "Undo"),
    ("CommandOrControl+Y", "Redo"),
    ("CommandOrControl+A", "Select All"),
    ("CommandOrControl+S", "Save"),
    ("CommandOrControl+P", "Print"),
    ("CommandOrControl+F", "Find"),
    ("CommandOrControl+W", "Close Window/Tab"),
    ("CommandOrControl+Q", "Quit App"),
    ("CommandOrControl+N", "New Window/File"),
    ("CommandOrControl+O", "Open File"),
    ("CommandOrControl+T", "New Tab"),
    ("CommandOrControl+R", "Refresh"),
    ("CommandOrControl+Shift+Z", "Redo"),
    ("CommandOrControl+Shift+T", "Reopen Tab"),
    ("CommandOrControl+Shift+N", "New Window (Private)"),
    ("Alt+Tab", "Switch Windows"),
    ("Alt+F4", "Close Window"),
    ("CommandOrControl+Tab", "Switch Tabs"),
    ("CommandOrControl+Shift+Tab", "Switch Tabs (Reverse)"),
];

fn common_key_display(key: &str) -> Option<&'static str> {
    Some(match key {
        "Enter" | "Return" => "↵",
        "Escape" => "Esc",
        "Backspace" => "⌫",
        "Delete" => "Del",
        "Tab" => "Tab",
        "Space" => "Space",
        "ArrowUp" => "↑",
        "ArrowDown" => "↓",
        "ArrowLeft" => "←",
        "ArrowRight" => "→",
        _ => return None,
    })
}

fn normalize_key(key: &str) -> String {
    if key.chars().count() == 1 {
        key.to_uppercase()
    } else {
        key.to_string()
    }
}

impl Shortcut {
    /// Parses an accelerator string. Platform-specific modifier spellings
    /// (`Command`, `Control`, `Ctrl`, `Super`, `Meta`, `Option`) fold into
    /// the cross-platform ones. `None` for an empty string or more than one
    /// non-modifier key. A modifier-only string parses with an empty key.
    pub fn parse(accelerator: &str) -> Option<Self> {
        if accelerator.is_empty() {
            return None;
        }
        let mut shortcut = Shortcut::default();
        for part in accelerator.split('+') {
            match part {
                "CommandOrControl" | "CmdOrCtrl" | "Command" | "Cmd" | "Control" | "Ctrl"
                | "Super" | "Meta" => shortcut.modifiers.cmd_or_ctrl = true,
                "Alt" | "Option" => shortcut.modifiers.alt = true,
                "Shift" => shortcut.modifiers.shift = true,
                "" => {}
                key if shortcut.key.is_empty() => shortcut.key = normalize_key(key),
                _ => return None,
            }
        }
        Some(shortcut)
    }

    /// Builds a shortcut from a key press. `key` uses DOM `KeyboardEvent.key`
    /// names. `None` for a bare modifier press or a key without modifiers.
    pub fn from_key_event(modifiers: Modifiers, key: &str) -> Option<Self> {
        if matches!(key, "Control" | "Meta" | "Alt" | "Shift") || key.is_empty() || !modifiers.any()
        {
            return None;
        }
        Some(Self {
            modifiers,
            key: normalize_key(key),
        })
    }

    pub fn is_valid(&self) -> bool {
        self.modifiers.any() && !self.key.is_empty()
    }

    pub fn validate(&self) -> ShortcutValidation {
        if !self.is_valid() {
            return ShortcutValidation::Invalid;
        }
        let accelerator = self.to_string();
        CONFLICTS
            .iter()
            .find(|(combo, _)| *combo == accelerator)
            .map_or(ShortcutValidation::Ok, |&(_, action)| {
                ShortcutValidation::Conflict(action)
            })
    }

    /// Key caps for display: `["⌘", "⇧", "V"]` on macOS, `["Ctrl", "Shift", "V"]`
    /// elsewhere.
    pub fn format_keys(&self, platform_is_mac: bool) -> Vec<String> {
        let (cmd, alt, shift) = if platform_is_mac {
            ("⌘", "⌥", "⇧")
        } else {
            ("Ctrl", "Alt", "Shift")
        };
        let mut keys = Vec::new();
        if self.modifiers.cmd_or_ctrl {
            keys.push(cmd.to_string());
        }
        if self.modifiers.alt {
            keys.push(alt.to_string());
        }
        if self.modifiers.shift {
            keys.push(shift.to_string());
        }
        if !self.key.is_empty() {
            keys.push(
                common_key_display(&self.key)
                    .map(str::to_string)
                    .unwrap_or_else(|| self.key.to_uppercase()),
            );
        }
        keys
    }

    /// `⌘⇧V` on macOS, `Ctrl + Shift + V` elsewhere.
    pub fn format(&self, platform_is_mac: bool) -> String {
        self.format_keys(platform_is_mac)
            .join(if platform_is_mac { "" } else { " + " })
    }

    /// Spelled-out form for tooltips: `Cmd + Shift + V` / `Ctrl + Shift + V`.
    pub fn format_verbose(&self, platform_is_mac: bool) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.modifiers.cmd_or_ctrl {
            parts.push(if platform_is_mac { "Cmd" } else { "Ctrl" });
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if !self.key.is_empty() {
            parts.push(&self.key);
        }
        parts.join(" + ")
    }
}

/// Serializes to the canonical accelerator string: modifiers in
/// `CommandOrControl`, `Alt`, `Shift` order, then the key.
impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts: Vec<&str> = Vec::new();
        if self.modifiers.cmd_or_ctrl {
            parts.push("CommandOrControl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if !self.key.is_empty() {
            parts.push(&self.key);
        }
        f.write_str(&parts.join("+"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mods(cmd_or_ctrl: bool, alt: bool, shift: bool) -> Modifiers {
        Modifiers {
            cmd_or_ctrl,
            alt,
            shift,
        }
    }

    #[test]
    fn parses_and_serializes_canonically() {
        let s = Shortcut::parse("CommandOrControl+Shift+V").unwrap();
        assert_eq!(s.modifiers, mods(true, false, true));
        assert_eq!(s.key, "V");
        assert_eq!(s.to_string(), "CommandOrControl+Shift+V");
        assert_eq!(
            Shortcut::parse("Shift+Alt+CommandOrControl+k")
                .unwrap()
                .to_string(),
            "CommandOrControl+Alt+Shift+K"
        );
    }

    #[test]
    fn folds_platform_modifier_aliases() {
        assert_eq!(
            Shortcut::parse("Ctrl+Option+F1").unwrap().to_string(),
            "CommandOrControl+Alt+F1"
        );
        assert_eq!(
            Shortcut::parse("Command+A").unwrap().to_string(),
            "CommandOrControl+A"
        );
    }

    #[test]
    fn rejects_empty_and_multiple_keys() {
        assert_eq!(Shortcut::parse(""), None);
        assert_eq!(Shortcut::parse("CommandOrControl+A+B"), None);
    }

    #[test]
    fn validity_needs_modifier_and_key() {
        assert!(
            Shortcut::parse("CommandOrControl+Shift+V")
                .unwrap()
                .is_valid()
        );
        assert!(!Shortcut::parse("V").unwrap().is_valid());
        assert!(
            !Shortcut::parse("CommandOrControl+Shift")
                .unwrap()
                .is_valid()
        );
        assert!(!Shortcut::default().is_valid());
    }

    #[test]
    fn validates_conflicts() {
        let v = |s: &str| Shortcut::parse(s).unwrap().validate();
        assert_eq!(v("CommandOrControl+Shift+V"), ShortcutValidation::Ok);
        assert_eq!(
            v("CommandOrControl+V"),
            ShortcutValidation::Conflict("Paste")
        );
        assert_eq!(
            v("Shift+CommandOrControl+Z"),
            ShortcutValidation::Conflict("Redo")
        );
        assert_eq!(v("Alt+F4"), ShortcutValidation::Conflict("Close Window"));
        assert_eq!(
            v("CommandOrControl+c"),
            ShortcutValidation::Conflict("Copy")
        );
        assert_eq!(v("Shift"), ShortcutValidation::Invalid);
    }

    #[test]
    fn validation_messages() {
        assert_eq!(ShortcutValidation::Ok.message(), None);
        assert_eq!(
            ShortcutValidation::Invalid.message().as_deref(),
            Some("Must have a modifier key")
        );
        assert_eq!(
            ShortcutValidation::Conflict("Paste").message().as_deref(),
            Some("Usually \"Paste\" — will be overridden")
        );
        assert!(ShortcutValidation::Conflict("Paste").is_valid());
        assert!(!ShortcutValidation::Invalid.is_valid());
    }

    #[test]
    fn formats_for_mac() {
        let s = Shortcut::parse("CommandOrControl+Alt+Shift+Enter").unwrap();
        assert_eq!(s.format_keys(true), vec!["⌘", "⌥", "⇧", "↵"]);
        assert_eq!(s.format(true), "⌘⌥⇧↵");
    }

    #[test]
    fn formats_for_windows() {
        let s = Shortcut::parse("CommandOrControl+Shift+V").unwrap();
        assert_eq!(s.format_keys(false), vec!["Ctrl", "Shift", "V"]);
        assert_eq!(s.format(false), "Ctrl + Shift + V");
        let s = Shortcut::parse("Alt+ArrowUp").unwrap();
        assert_eq!(s.format(false), "Alt + ↑");
        let s = Shortcut::parse("Alt+Escape").unwrap();
        assert_eq!(s.format(false), "Alt + Esc");
        let s = Shortcut::parse("Alt+PageUp").unwrap();
        assert_eq!(s.format(false), "Alt + PAGEUP");
    }

    #[test]
    fn formats_verbose() {
        let s = Shortcut::parse("CommandOrControl+Shift+V").unwrap();
        assert_eq!(s.format_verbose(true), "Cmd + Shift + V");
        assert_eq!(s.format_verbose(false), "Ctrl + Shift + V");
    }

    #[test]
    fn builds_from_key_events() {
        assert_eq!(
            Shortcut::from_key_event(mods(true, false, true), "v")
                .unwrap()
                .to_string(),
            "CommandOrControl+Shift+V"
        );
        assert_eq!(
            Shortcut::from_key_event(mods(false, true, false), "F4")
                .unwrap()
                .to_string(),
            "Alt+F4"
        );
        assert_eq!(
            Shortcut::from_key_event(mods(true, false, false), "Shift"),
            None
        );
        assert_eq!(Shortcut::from_key_event(Modifiers::default(), "a"), None);
    }
}
