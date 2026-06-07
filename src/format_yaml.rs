//! YAML formatter — parse and re-serialize with consistent indentation.

/// Format YAML content: parse then re-serialize with 2-space indent.
pub fn format_yaml(content: &str) -> Result<String, String> {
    let value: serde_yaml::Value = serde_yaml::from_str(content).map_err(|e| e.to_string())?;
    serde_yaml::to_string(&value).map_err(|e| e.to_string())
}
