//! Build system auto-detection from workspace marker files.

use std::path::Path;

/// Detected build system with associated commands.
pub struct BuildSystem {
    pub(crate) build: &'static str,
    pub(crate) test: &'static str,
    pub(crate) test_file: Option<&'static str>,
}

/// Detect build system by scanning workspace root for marker files.
/// Returns the first match in priority order.
pub fn detect(root: &Path) -> Option<BuildSystem> {
    let markers: &[(&[&str], BuildSystem)] = &[
        (
            &["Makefile", "GNUmakefile"],
            BuildSystem {
                build: "make",
                test: "make test",
                test_file: None,
            },
        ),
        (
            &["Cargo.toml"],
            BuildSystem {
                build: "cargo build",
                test: "cargo test --workspace",
                test_file: Some("cargo test --lib {file}"),
            },
        ),
        (
            &["go.mod"],
            BuildSystem {
                build: "go build ./...",
                test: "go test ./...",
                test_file: Some("go test ./{dir}"),
            },
        ),
        (
            &["gradlew"],
            BuildSystem {
                build: "./gradlew build",
                test: "./gradlew test",
                test_file: None,
            },
        ),
        (
            &["build.gradle", "build.gradle.kts"],
            BuildSystem {
                build: "gradle build",
                test: "gradle test",
                test_file: None,
            },
        ),
        (
            &["pom.xml"],
            BuildSystem {
                build: "mvn compile",
                test: "mvn test",
                test_file: None,
            },
        ),
        (
            &["CMakeLists.txt"],
            BuildSystem {
                build: "cmake --build build",
                test: "ctest --test-dir build",
                test_file: None,
            },
        ),
        (
            &["package.json"],
            BuildSystem {
                build: "npm run build",
                test: "npm test",
                test_file: None,
            },
        ),
        (
            &["build.xml"],
            BuildSystem {
                build: "ant",
                test: "ant test",
                test_file: None,
            },
        ),
        (
            &["configure.ac", "Makefile.am"],
            BuildSystem {
                build: "make",
                test: "make check",
                test_file: None,
            },
        ),
        (
            &["meson.build"],
            BuildSystem {
                build: "meson compile -C build",
                test: "meson test -C build",
                test_file: None,
            },
        ),
        (
            &["BUILD", "WORKSPACE"],
            BuildSystem {
                build: "bazel build //...",
                test: "bazel test //...",
                test_file: None,
            },
        ),
    ];

    for (files, system) in markers {
        for file in *files {
            if root.join(file).exists() {
                return Some(BuildSystem {
                    build: system.build,
                    test: system.test,
                    test_file: system.test_file,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detect_cargo() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let bs = detect(dir.path()).unwrap();
        assert_eq!(bs.build, "cargo build");
        assert_eq!(bs.test, "cargo test --workspace");
    }

    #[test]
    fn detect_makefile_priority() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Makefile"), "").unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let bs = detect(dir.path()).unwrap();
        assert_eq!(bs.build, "make");
    }

    #[test]
    fn detect_none() {
        let dir = TempDir::new().unwrap();
        assert!(detect(dir.path()).is_none());
    }
}
