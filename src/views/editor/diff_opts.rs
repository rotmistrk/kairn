//! Options parsed from :diff args.

/// Options parsed from :diff args.
pub struct DiffOpts {
    pub(crate) base: String,
    pub(crate) context: usize,
    pub(crate) ignore_ws: bool,
    pub(crate) side_by_side: bool,
}

impl DiffOpts {
    pub fn new(base: &str, context: usize, ignore_ws: bool, side_by_side: bool) -> Self {
        Self {
            base: base.to_string(),
            context,
            ignore_ws,
            side_by_side,
        }
    }
    pub fn base(&self) -> &str {
        &self.base
    }
    pub fn context(&self) -> usize {
        self.context
    }
    pub fn ignore_ws(&self) -> bool {
        self.ignore_ws
    }
    pub fn side_by_side(&self) -> bool {
        self.side_by_side
    }
}

pub fn parse_diff_args(args: &str) -> DiffOpts {
    let mut base = "HEAD".to_string();
    let mut context = 2;
    let mut ignore_ws = false;
    let mut side_by_side = false;
    for arg in args.split_whitespace() {
        if arg == "-w" {
            ignore_ws = true;
        } else if arg == "-y" {
            side_by_side = true;
        } else if let Some(n) = arg.strip_prefix("-U") {
            context = n.parse().unwrap_or(3);
        } else if !arg.starts_with('-') {
            base = arg.to_string();
        }
    }
    DiffOpts {
        base,
        context,
        ignore_ws,
        side_by_side,
    }
}
