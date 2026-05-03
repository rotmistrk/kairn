# src/config/defaults.tcl — compiled into binary via include_str!
# Defines the kairn context with all settings and default keybindings.

context kairn {
    declare keymap       : enum {vi emacs classic}
    declare tab-width    : int
    declare line-numbers : bool
    declare auto-save    : bool
    declare theme        : string
    declare shell        : string
    declare kiro-command : string

    set keymap       vi
    set tab-width    4
    set line-numbers true
    set auto-save    false
    set theme        "gruvbox-dark"
    set shell        ""
    set kiro-command "kiro-cli"
}

# Default keybindings
bind "ctrl+q"           { editor quit }
bind "ctrl+s"           { buffer save }
bind "ctrl+b"           { window toggle-tree }
bind "ctrl+tab"         { window cycle-focus }
bind "ctrl+p"           { editor open-search }
bind "ctrl+d"           { editor diff }
bind "ctrl+g"           { editor git-log }
bind "ctrl+shift+down"  { editor cycle-mode next }
bind "ctrl+shift+up"    { editor cycle-mode prev }
bind "f1"               { editor help }
bind "f3"               { window focus tree }
bind "f4"               { window focus editor }
bind "f5"               { window focus terminal }
bind "f6"               { window toggle-left }
bind "f11"              { window refresh-tree }
bind "f12"              { window redraw }
bind "alt+left"         { terminal prev-tab }
bind "alt+right"        { terminal next-tab }
bind "ctrl+x n"         { terminal new-kiro }
bind "ctrl+x t"         { terminal new-shell }
bind "ctrl+x k"         { terminal close-tab }
bind "ctrl+x ctrl+s"    { buffer save }
bind "ctrl+x s"         { editor save-session }

# Default theme
theme load "gruvbox-dark"
