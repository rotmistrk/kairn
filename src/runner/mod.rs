//! Build and test runners: project detection, error parsing, test results.

pub mod tests;

use std::path::Path;

/// Detected project type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Cargo,
    Go,
    Gradle,
    Maven,
    Npm,
}

impl ProjectType {
    /// Build command for this project type.
    pub fn build_command(self) -> &'static str {
        match self {
            Self::Cargo => "cargo build",
            Self::Go => "go build ./...",
            Self::Gradle => "./gradlew build",
            Self::Maven => "mvn compile",
            Self::Npm => "npm run build",
        }
    }

    /// Test command for this project type.
    pub fn test_command(self) -> &'static str {
        match self {
            Self::Cargo => "cargo test",
            Self::Go => "go test ./...",
            Self::Gradle => "./gradlew test",
            Self::Maven => "mvn test",
            Self::Npm => "npm test",
        }
    }
}

/// Detect project type from workspace root.
pub fn detect_project(workspace: &Path) -> Option<ProjectType> {
    if workspace.join("Cargo.toml").exists() {
        Some(ProjectType::Cargo)
    } else if workspace.join("go.mod").exists() {
        Some(ProjectType::Go)
    } else if workspace.join("gradlew").exists() {
        Some(ProjectType::Gradle)
    } else if workspace.join("pom.xml").exists() {
        Some(ProjectType::Maven)
    } else if workspace.join("package.json").exists() {
        Some(ProjectType::Npm)
    } else {
        None
    }
}

/// Severity of a build error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A parsed compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub severity: Severity,
}

/// Parse compiler output into structured errors.
pub fn parse_errors(output: &str, project: ProjectType) -> Vec<BuildError> {
    match project {
        ProjectType::Cargo => parse_rustc(output),
        ProjectType::Go => parse_go(output),
        ProjectType::Npm => parse_tsc(output),
        ProjectType::Gradle | ProjectType::Maven => parse_javac(output),
    }
}

/// Parse rustc/cargo error output.
/// Format: `error[E0308]: ... --> src/main.rs:10:5`
fn parse_rustc(output: &str) -> Vec<BuildError> {
    let re = match regex::Regex::new(
        r"(?m)^(error|warning)(?:\[E\d+\])?: (.+)\n\s*--> (.+):(\d+):(\d+)",
    ) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    re.captures_iter(output)
        .map(|cap| {
            let severity = if &cap[1] == "error" {
                Severity::Error
            } else {
                Severity::Warning
            };
            BuildError {
                file: cap[3].to_string(),
                line: cap[4].parse().unwrap_or(1),
                col: cap[5].parse().unwrap_or(1),
                message: cap[2].to_string(),
                severity,
            }
        })
        .collect()
}

/// Parse Go compiler output.
/// Format: `./main.go:10:5: error message`
fn parse_go(output: &str) -> Vec<BuildError> {
    let re = match regex::Regex::new(r"(?m)^(.+\.go):(\d+):(\d+): (.+)$") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    re.captures_iter(output)
        .map(|cap| BuildError {
            file: cap[1].to_string(),
            line: cap[2].parse().unwrap_or(1),
            col: cap[3].parse().unwrap_or(1),
            message: cap[4].to_string(),
            severity: Severity::Error,
        })
        .collect()
}

/// Parse TypeScript compiler output.
/// Format: `src/index.ts(10,5): error TS2322: message`
fn parse_tsc(output: &str) -> Vec<BuildError> {
    let re = match regex::Regex::new(r"(?m)^(.+)\((\d+),(\d+)\): (error|warning) TS\d+: (.+)$") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    re.captures_iter(output)
        .map(|cap| {
            let severity = if &cap[4] == "error" {
                Severity::Error
            } else {
                Severity::Warning
            };
            BuildError {
                file: cap[1].to_string(),
                line: cap[2].parse().unwrap_or(1),
                col: cap[3].parse().unwrap_or(1),
                message: cap[5].to_string(),
                severity,
            }
        })
        .collect()
}

/// Parse javac output.
/// Format: `src/Main.java:10: error: message`
fn parse_javac(output: &str) -> Vec<BuildError> {
    let re = match regex::Regex::new(r"(?m)^(.+\.java):(\d+): (error|warning): (.+)$") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    re.captures_iter(output)
        .map(|cap| {
            let severity = if &cap[3] == "error" {
                Severity::Error
            } else {
                Severity::Warning
            };
            BuildError {
                file: cap[1].to_string(),
                line: cap[2].parse().unwrap_or(1),
                col: 1,
                message: cap[4].to_string(),
                severity,
            }
        })
        .collect()
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detect_cargo_project() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_project(tmp.path()), Some(ProjectType::Cargo));
    }

    #[test]
    fn detect_go_project() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("go.mod"), "").unwrap();
        assert_eq!(detect_project(tmp.path()), Some(ProjectType::Go));
    }

    #[test]
    fn detect_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(detect_project(tmp.path()), None);
    }

    #[test]
    fn parse_rustc_errors() {
        let output = r#"error[E0308]: mismatched types
 --> src/main.rs:10:5
  |
10 |     let x: u32 = "hello";
  |                   ^^^^^^^ expected `u32`, found `&str`

warning: unused variable: `y`
 --> src/lib.rs:3:9
  |
3 |     let y = 42;
  |         ^ help: if this is intentional, prefix it with an underscore: `_y`
"#;
        let errors = parse_rustc(output);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].file, "src/main.rs");
        assert_eq!(errors[0].line, 10);
        assert_eq!(errors[0].col, 5);
        assert_eq!(errors[0].severity, Severity::Error);
        assert_eq!(errors[1].severity, Severity::Warning);
    }

    #[test]
    fn parse_go_errors() {
        let output = "./main.go:15:2: undefined: foo\n./util.go:3:10: cannot use x\n";
        let errors = parse_go(output);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].file, "./main.go");
        assert_eq!(errors[0].line, 15);
    }

    #[test]
    fn parse_tsc_errors() {
        let output = "src/index.ts(10,5): error TS2322: Type 'string' is not assignable\n";
        let errors = parse_tsc(output);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "src/index.ts");
        assert_eq!(errors[0].line, 10);
        assert_eq!(errors[0].col, 5);
    }

    #[test]
    fn parse_javac_errors() {
        let output = "src/Main.java:10: error: cannot find symbol\n";
        let errors = parse_javac(output);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "src/Main.java");
    }
}
