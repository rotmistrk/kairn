use super::*;

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

#[test]
fn split_abbreviations() {
    assert_eq!(lookup_command("sp"), Some(ExCmdId::Split));
    assert_eq!(lookup_command("spl"), Some(ExCmdId::Split));
    assert_eq!(lookup_command("split"), Some(ExCmdId::Split));
    assert_eq!(lookup_command("vs"), Some(ExCmdId::Vsplit));
    assert_eq!(lookup_command("vsp"), Some(ExCmdId::Vsplit));
    assert_eq!(lookup_command("vsplit"), Some(ExCmdId::Vsplit));
    assert_eq!(lookup_command("on"), Some(ExCmdId::Only));
    assert_eq!(lookup_command("only"), Some(ExCmdId::Only));
}

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
    assert_eq!(lookup_command("no"), None);
}

#[test]
fn se_vs_setglobal() {
    assert_eq!(lookup_command("se"), Some(ExCmdId::Set));
    assert_eq!(lookup_command("set"), Some(ExCmdId::Set));
    assert_eq!(lookup_command("setg"), Some(ExCmdId::SetGlobal));
}

#[test]
fn w_vs_wq() {
    assert_eq!(lookup_command("w"), Some(ExCmdId::Write));
}

#[test]
fn e_is_edit_not_exit() {
    assert_eq!(lookup_command("e"), Some(ExCmdId::Edit));
    assert_eq!(lookup_command("x"), Some(ExCmdId::Exit));
}

#[test]
fn split_cmd_word_basic() {
    assert_eq!(split_cmd_word("diff HEAD"), ("diff", " HEAD"));
    assert_eq!(split_cmd_word("d"), ("d", ""));
    assert_eq!(split_cmd_word("s/foo/bar/"), ("s", "/foo/bar/"));
    assert_eq!(split_cmd_word(""), ("", ""));
}
