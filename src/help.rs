//! Help text generator for kairn (legacy — delegates to help_topics).

use crate::help_topics::generate_topic;

/// Generate the full help text (overview topic).
pub fn help_text() -> String {
    generate_topic("", &[])
}
