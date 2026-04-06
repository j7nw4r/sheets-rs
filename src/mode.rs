use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Select,
    Command,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Select => write!(f, "VISUAL"),
            Mode::Command => write!(f, "COMMAND"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    None,
    Command,
    SearchForward,
    SearchBackward,
}

impl PromptKind {
    pub fn prefix(self) -> &'static str {
        match self {
            PromptKind::None => "",
            PromptKind::Command => ":",
            PromptKind::SearchForward => "/",
            PromptKind::SearchBackward => "?",
        }
    }
}

/// Represents a pending multi-key action in normal mode.
/// Only one can be active at a time (replaces Go's 8+ boolean flags).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    None,
    /// Accumulating a goto target (g prefix)
    Goto,
    /// Pending delete (d prefix)
    Delete,
    /// Pending yank (y prefix)
    Yank,
    /// Pending z-scroll command
    ZScroll,
    /// Waiting for register name (" prefix)
    Register,
    /// Setting a mark (m prefix)
    Mark,
    /// Jumping to a mark (' or ` prefix)
    MarkJump { exact: bool },
}
