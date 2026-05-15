//! Vim-style ex command table with unambiguous prefix matching.
//!
//! Each command has a full name and a minimum abbreviation. A user input
//! matches if it is a prefix of the full name AND at least as long as the
//! minimum abbreviation.

/// Known ex command identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExCmdId {
    Delete,
    Diff,
    Edit,
    NoDiff,
    NoHighlight,
    Quit,
    QuitForce,
    Set,
    SetGlobal,
    Substitute,
    Write,
    WriteQuit,
    Exit,
    Yank,
}

struct CmdEntry {
    full: &'static str,
    min_abbrev: usize, // minimum chars required
    id: ExCmdId,
}

/// The command table. Ordered by full name for clarity.
const CMD_TABLE: &[CmdEntry] = &[
    CmdEntry {
        full: "delete",
        min_abbrev: 1,
        id: ExCmdId::Delete,
    },
    CmdEntry {
        full: "diff",
        min_abbrev: 3,
        id: ExCmdId::Diff,
    },
    CmdEntry {
        full: "edit",
        min_abbrev: 1,
        id: ExCmdId::Edit,
    },
    CmdEntry {
        full: "nodiff",
        min_abbrev: 3,
        id: ExCmdId::NoDiff,
    },
    CmdEntry {
        full: "nohlsearch",
        min_abbrev: 3,
        id: ExCmdId::NoHighlight,
    },
    CmdEntry {
        full: "quit",
        min_abbrev: 1,
        id: ExCmdId::Quit,
    },
    CmdEntry {
        full: "set",
        min_abbrev: 2,
        id: ExCmdId::Set,
    },
    CmdEntry {
        full: "setglobal",
        min_abbrev: 4,
        id: ExCmdId::SetGlobal,
    },
    CmdEntry {
        full: "substitute",
        min_abbrev: 1,
        id: ExCmdId::Substitute,
    },
    CmdEntry {
        full: "write",
        min_abbrev: 1,
        id: ExCmdId::Write,
    },
    CmdEntry {
        full: "wq",
        min_abbrev: 2,
        id: ExCmdId::WriteQuit,
    },
    CmdEntry {
        full: "x",
        min_abbrev: 1,
        id: ExCmdId::Exit,
    },
    CmdEntry {
        full: "yank",
        min_abbrev: 1,
        id: ExCmdId::Yank,
    },
];

/// Look up a command name by unambiguous prefix.
/// Returns None if no match or ambiguous.
pub fn lookup_command(input: &str) -> Option<ExCmdId> {
    if input.is_empty() {
        return None;
    }
    // Special cases that aren't alpha-only
    if input == "q!" {
        return Some(ExCmdId::QuitForce);
    }
    let mut found: Option<ExCmdId> = None;
    for entry in CMD_TABLE {
        if input.len() >= entry.min_abbrev && input.len() <= entry.full.len() && entry.full.starts_with(input) {
            if found.is_some() {
                return None; // ambiguous
            }
            found = Some(entry.id);
        }
    }
    found
}

/// Extract the command word (leading alphabetic chars) from an ex command string.
/// Returns (command_word, rest) where rest is everything after the command word.
pub fn split_cmd_word(input: &str) -> (&str, &str) {
    let end = input.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(input.len());
    (&input[..end], &input[end..])
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Exact and minimum abbreviation tests ---

    #[test]
    fn delete_abbreviations() {
        assert_eq!(lookup_command("d"), Some(ExCmdId::Delete));
        assert_eq!(lookup_command("de"), Some(ExCmdId::Delete));
        assert_eq!(lookup_command("del"), Some(ExCmdId::Delete));
        assert_eq!(lookup_command("dele"), Some(ExCmdId::Delete));
        assert_eq!(lookup_command("delet"), Some(ExCmdId::Delete));
        assert_eq!(lookup_command("delete"), Some(ExCmdId::Delete));
    }

    #[test]
    fn diff_abbreviations() {
        assert_eq!(lookup_command("dif"), Some(ExCmdId::Diff));
        assert_eq!(lookup_command("diff"), Some(ExCmdId::Diff));
    }

    #[test]
    fn di_is_no_match() {
        // "di" is NOT a prefix of "delete" (de...) and "diff" requires min 3
        assert_eq!(lookup_command("di"), None);
    }

    #[test]
    fn edit_abbreviations() {
        assert_eq!(lookup_command("e"), Some(ExCmdId::Edit));
        assert_eq!(lookup_command("ed"), Some(ExCmdId::Edit));
        assert_eq!(lookup_command("edi"), Some(ExCmdId::Edit));
        assert_eq!(lookup_command("edit"), Some(ExCmdId::Edit));
    }

    #[test]
    fn nodiff_abbreviations() {
        assert_eq!(lookup_command("nod"), Some(ExCmdId::NoDiff));
        assert_eq!(lookup_command("nodi"), Some(ExCmdId::NoDiff));
        assert_eq!(lookup_command("nodif"), Some(ExCmdId::NoDiff));
        assert_eq!(lookup_command("nodiff"), Some(ExCmdId::NoDiff));
    }

    #[test]
    fn quit_abbreviations() {
        assert_eq!(lookup_command("q"), Some(ExCmdId::Quit));
        assert_eq!(lookup_command("qu"), Some(ExCmdId::Quit));
        assert_eq!(lookup_command("qui"), Some(ExCmdId::Quit));
        assert_eq!(lookup_command("quit"), Some(ExCmdId::Quit));
        assert_eq!(lookup_command("q!"), Some(ExCmdId::QuitForce));
    }

    #[test]
    fn set_abbreviations() {
        assert_eq!(lookup_command("se"), Some(ExCmdId::Set));
        assert_eq!(lookup_command("set"), Some(ExCmdId::Set));
    }

    #[test]
    fn setglobal_abbreviations() {
        assert_eq!(lookup_command("setg"), Some(ExCmdId::SetGlobal));
        assert_eq!(lookup_command("setgl"), Some(ExCmdId::SetGlobal));
        assert_eq!(lookup_command("setglobal"), Some(ExCmdId::SetGlobal));
    }

    #[test]
    fn substitute_abbreviations() {
        assert_eq!(lookup_command("s"), Some(ExCmdId::Substitute));
        assert_eq!(lookup_command("su"), Some(ExCmdId::Substitute));
        assert_eq!(lookup_command("sub"), Some(ExCmdId::Substitute));
        assert_eq!(lookup_command("substitute"), Some(ExCmdId::Substitute));
    }

    #[test]
    fn s_is_not_ambiguous_with_set() {
        // "s" → substitute (min 1), "set" (min 2) requires 2 chars
        assert_eq!(lookup_command("s"), Some(ExCmdId::Substitute));
    }

    #[test]
    fn write_abbreviations() {
        assert_eq!(lookup_command("w"), Some(ExCmdId::Write));
        assert_eq!(lookup_command("wr"), Some(ExCmdId::Write));
        assert_eq!(lookup_command("wri"), Some(ExCmdId::Write));
        assert_eq!(lookup_command("write"), Some(ExCmdId::Write));
    }

    #[test]
    fn wq_exact() {
        assert_eq!(lookup_command("wq"), Some(ExCmdId::WriteQuit));
    }

    #[test]
    fn exit_abbreviation() {
        assert_eq!(lookup_command("x"), Some(ExCmdId::Exit));
    }

    #[test]
    fn yank_abbreviations() {
        assert_eq!(lookup_command("y"), Some(ExCmdId::Yank));
        assert_eq!(lookup_command("ya"), Some(ExCmdId::Yank));
        assert_eq!(lookup_command("yan"), Some(ExCmdId::Yank));
        assert_eq!(lookup_command("yank"), Some(ExCmdId::Yank));
    }

    // --- Ambiguity / no-match tests ---

    #[test]
    fn unknown_command() {
        assert_eq!(lookup_command("foo"), None);
        assert_eq!(lookup_command("z"), None);
    }

    #[test]
    fn empty_input() {
        assert_eq!(lookup_command(""), None);
    }

    #[test]
    fn no_is_ambiguous_with_nodiff() {
        // "no" is 2 chars, nodiff requires 3 → no match
        assert_eq!(lookup_command("no"), None);
    }

    #[test]
    fn se_vs_setglobal() {
        // "se" matches "set" (min 2) but not "setglobal" (min 4)
        assert_eq!(lookup_command("se"), Some(ExCmdId::Set));
        // "set" matches "set" (len == full.len()) but not "setglobal" (min 4)
        assert_eq!(lookup_command("set"), Some(ExCmdId::Set));
        // "setg" matches only "setglobal" (min 4), not "set" (len > full.len())
        assert_eq!(lookup_command("setg"), Some(ExCmdId::SetGlobal));
    }

    #[test]
    fn w_vs_wq() {
        // "w" matches "write" (min 1) but not "wq" (min 2)
        assert_eq!(lookup_command("w"), Some(ExCmdId::Write));
    }

    #[test]
    fn e_is_edit_not_exit() {
        // "x" is its own command, "e" is unambiguously "edit"
        assert_eq!(lookup_command("e"), Some(ExCmdId::Edit));
        assert_eq!(lookup_command("x"), Some(ExCmdId::Exit));
    }

    // --- split_cmd_word tests ---

    #[test]
    fn split_simple() {
        assert_eq!(split_cmd_word("diff HEAD"), ("diff", " HEAD"));
        assert_eq!(split_cmd_word("d"), ("d", ""));
        assert_eq!(split_cmd_word("s/foo/bar/"), ("s", "/foo/bar/"));
        assert_eq!(split_cmd_word(""), ("", ""));
    }
}
