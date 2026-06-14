//! Tcl bridge for `mcp-permission` command.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use crate::mcp::permissions::{Permission, PermissionTable};

pub fn register(interp: &mut Interpreter, table: Arc<Mutex<PermissionTable>>) {
    interp.register_fn("mcp-permission", move |_interp, args| {
        let tool = super::arg_str(args, 0)?;
        let perm_str = super::arg_str(args, 1)?;
        let perm = match perm_str.as_str() {
            "allow" => Permission::Allow,
            "confirm" => Permission::Confirm,
            "deny" => Permission::Deny,
            other => return Err(TclError::new(format!("bad permission: {other} (allow|confirm|deny)"))),
        };
        let mut t = table.lock().map_err(|e| TclError::new(e.to_string()))?;
        if tool == "*write" {
            t.set_default_write(perm);
        } else if tool == "*read" {
            t.set_default_read(perm);
        } else {
            t.set(&tool, perm);
        }
        Ok(TclValue::Str(String::new()))
    });
}
