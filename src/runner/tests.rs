//! Test runner: parse test output into pass/fail results.

use super::ProjectType;

/// Result of a single test case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    Pass,
    Fail,
    Skip,
}

/// A single test result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    /// Optional source file location.
    pub file: Option<String>,
    /// Optional line number.
    pub line: Option<usize>,
}

/// A test suite (group of tests).
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub results: Vec<TestResult>,
}

impl TestSuite {
    /// Count passed tests.
    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Pass)
            .count()
    }

    /// Count failed tests.
    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Fail)
            .count()
    }

    /// Total tests.
    pub fn total(&self) -> usize {
        self.results.len()
    }
}

/// Parse test output into suites based on project type.
pub fn parse_test_output(output: &str, project: ProjectType) -> Vec<TestSuite> {
    match project {
        ProjectType::Cargo => parse_cargo_test(output),
        ProjectType::Go => parse_go_test(output),
        _ => parse_generic(output),
    }
}

/// Parse `cargo test` output.
/// Format: `test module::name ... ok` or `test module::name ... FAILED`
fn parse_cargo_test(output: &str) -> Vec<TestSuite> {
    let re = match regex::Regex::new(r"(?m)^test (.+) \.\.\. (ok|FAILED|ignored)$") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for cap in re.captures_iter(output) {
        let name = cap[1].to_string();
        let status = match &cap[2] {
            "ok" => TestStatus::Pass,
            "FAILED" => TestStatus::Fail,
            _ => TestStatus::Skip,
        };
        results.push(TestResult {
            name,
            status,
            file: None,
            line: None,
        });
    }

    if results.is_empty() {
        return Vec::new();
    }

    vec![TestSuite {
        name: "cargo test".to_string(),
        results,
    }]
}

/// Parse `go test` output.
/// Format: `--- PASS: TestName (0.00s)` or `--- FAIL: TestName (0.00s)`
fn parse_go_test(output: &str) -> Vec<TestSuite> {
    let re = match regex::Regex::new(r"(?m)^--- (PASS|FAIL|SKIP): (.+?) \(") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for cap in re.captures_iter(output) {
        let status = match &cap[1] {
            "PASS" => TestStatus::Pass,
            "FAIL" => TestStatus::Fail,
            _ => TestStatus::Skip,
        };
        results.push(TestResult {
            name: cap[2].to_string(),
            status,
            file: None,
            line: None,
        });
    }

    if results.is_empty() {
        return Vec::new();
    }

    vec![TestSuite {
        name: "go test".to_string(),
        results,
    }]
}

/// Generic fallback: look for common pass/fail patterns.
fn parse_generic(output: &str) -> Vec<TestSuite> {
    let pass_re = match regex::Regex::new(r"(?mi)^\s*[✓✔]\s+(.+)$") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let fail_re = match regex::Regex::new(r"(?mi)^\s*[✗✘×]\s+(.+)$") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for cap in pass_re.captures_iter(output) {
        results.push(TestResult {
            name: cap[1].trim().to_string(),
            status: TestStatus::Pass,
            file: None,
            line: None,
        });
    }
    for cap in fail_re.captures_iter(output) {
        results.push(TestResult {
            name: cap[1].trim().to_string(),
            status: TestStatus::Fail,
            file: None,
            line: None,
        });
    }

    if results.is_empty() {
        return Vec::new();
    }

    vec![TestSuite {
        name: "tests".to_string(),
        results,
    }]
}

/// Build the command string for running a single test.
pub fn single_test_command(project: ProjectType, test_name: &str) -> String {
    match project {
        ProjectType::Cargo => format!("cargo test {test_name} -- --exact"),
        ProjectType::Go => format!("go test -run {test_name} ./..."),
        ProjectType::Gradle => format!("./gradlew test --tests {test_name}"),
        ProjectType::Maven => {
            format!("mvn test -Dtest={test_name}")
        }
        ProjectType::Npm => format!("npm test -- --grep {test_name}"),
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn parse_cargo_test_output() {
        let output = "\
running 3 tests
test buffer::tests::insert ... ok
test buffer::tests::delete ... FAILED
test buffer::tests::undo ... ignored

test result: FAILED. 1 passed; 1 failed; 1 ignored;
";
        let suites = parse_cargo_test(output);
        assert_eq!(suites.len(), 1);
        assert_eq!(suites[0].results.len(), 3);
        assert_eq!(suites[0].passed(), 1);
        assert_eq!(suites[0].failed(), 1);
    }

    #[test]
    fn parse_go_test_output() {
        let output = "\
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSub
--- FAIL: TestSub (0.01s)
";
        let suites = parse_go_test(output);
        assert_eq!(suites.len(), 1);
        assert_eq!(suites[0].results.len(), 2);
        assert_eq!(suites[0].results[0].status, TestStatus::Pass);
        assert_eq!(suites[0].results[1].status, TestStatus::Fail);
    }

    #[test]
    fn single_test_cargo() {
        let cmd = single_test_command(ProjectType::Cargo, "my_test");
        assert_eq!(cmd, "cargo test my_test -- --exact");
    }

    #[test]
    fn single_test_go() {
        let cmd = single_test_command(ProjectType::Go, "TestFoo");
        assert_eq!(cmd, "go test -run TestFoo ./...");
    }
}
