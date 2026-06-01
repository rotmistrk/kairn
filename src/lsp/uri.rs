//! URI utilities — convert file:// URIs to filesystem paths.

/// Convert a file:// URI to a filesystem path, percent-decoding special characters.
pub fn uri_to_path(uri: &str) -> String {
    let path = uri.strip_prefix("file://").unwrap_or(uri);
    percent_decode(path)
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(val) = decode_hex(bytes[i + 1], bytes[i + 2]) {
                out.push(val);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn decode_hex(hi: u8, lo: u8) -> Option<u8> {
    let h = hex_val(hi)?;
    let l = hex_val(lo)?;
    Some(h << 4 | l)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_path_unchanged() {
        assert_eq!(uri_to_path("file:///home/user/src/main.rs"), "/home/user/src/main.rs");
    }

    #[test]
    fn plus_signs_decoded() {
        assert_eq!(
            uri_to_path("file:///home/user/kairn%2B%2B/src/main.cpp"),
            "/home/user/kairn++/src/main.cpp"
        );
    }

    #[test]
    fn spaces_decoded() {
        assert_eq!(
            uri_to_path("file:///home/user/my%20project/f.rs"),
            "/home/user/my project/f.rs"
        );
    }

    #[test]
    fn no_prefix_still_decodes() {
        assert_eq!(uri_to_path("/path/with%20space"), "/path/with space");
    }
}
