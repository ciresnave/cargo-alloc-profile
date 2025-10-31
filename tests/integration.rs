use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_basic_run() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Allocation Profile"),
        "Should contain report header"
    );
}

#[test]
fn test_json_output() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "-o",
            "json",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(
        json.get("allocations").is_some(),
        "Should have allocations field"
    );
    assert!(json.get("summary").is_some(), "Should have summary field");
}

#[test]
fn test_verbosity_levels() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "-v",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // With -v, should show KB values
    assert!(
        stdout.contains("KB"),
        "Should contain KB measurements with -v"
    );
}

#[test]
fn test_filter_option() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "-f",
            "realloc",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should only show allocations matching "realloc"
    assert!(stdout.contains("realloc"), "Should contain realloc");
}

#[test]
fn test_sort_by_size() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "--sort-by",
            "size",
            "-v",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse output to verify sorting (largest first)
    let lines: Vec<&str> = stdout.lines().filter(|l| l.contains("KB")).collect();

    if lines.len() >= 2 {
        // Extract KB values and verify descending order
        let kb_values: Vec<f64> = lines
            .iter()
            .filter_map(|line| {
                line.split("(")
                    .nth(1)
                    .and_then(|s| s.split(" KB").next())
                    .and_then(|s| s.parse().ok())
            })
            .collect();

        for i in 1..kb_values.len() {
            assert!(
                kb_values[i - 1] >= kb_values[i],
                "Should be sorted by size descending"
            );
        }
    }
}

#[test]
fn test_limit_option() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "--limit",
            "2",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Count allocation lines (those with ": number" pattern)
    let alloc_lines = stdout
        .lines()
        .filter(|l| l.contains(": ") && !l.starts_with(' '))
        .count();

    assert!(alloc_lines <= 2, "Should show at most 2 allocation sites");
}

#[test]
fn test_save_and_compare() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let baseline_path = temp_dir.path().join("baseline.json");

    // Save baseline
    let save_output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "--save",
            baseline_path.to_str().unwrap(),
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute save command");

    assert!(save_output.status.success(), "Save command should succeed");
    assert!(baseline_path.exists(), "Baseline file should be created");

    // Compare against baseline
    let compare_output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "--compare",
            baseline_path.to_str().unwrap(),
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute compare command");

    assert!(
        compare_output.status.success(),
        "Compare command should succeed"
    );
    let stdout = String::from_utf8_lossy(&compare_output.stdout);

    assert!(
        stdout.contains("Allocation Comparison"),
        "Should show comparison header"
    );
    assert!(stdout.contains("vs"), "Should show comparison format");
}

#[test]
fn test_min_count_filter() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "--min-count",
            "10",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse count values and verify all are >= 10
    for line in stdout.lines() {
        if let Some(count_str) = line.split(": ").nth(1) {
            if let Some(count) = count_str.split_whitespace().next() {
                if let Ok(count_val) = count.parse::<usize>() {
                    assert!(count_val >= 10, "All counts should be >= 10");
                }
            }
        }
    }
}

#[test]
fn test_group_by_module() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "alloc-profile",
            "--group-by",
            "module",
            "run",
            "--example",
            "simple",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // With module grouping, should see module-level aggregation
    // (fewer distinct entries than function-level)
    assert!(stdout.contains("::"), "Should show module paths");
}

#[test]
fn test_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "alloc-profile", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Help command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify key options are documented
    assert!(
        stdout.contains("--min-count"),
        "Should document --min-count"
    );
    assert!(stdout.contains("--sort-by"), "Should document --sort-by");
    assert!(stdout.contains("--filter"), "Should document --filter");
    assert!(stdout.contains("--save"), "Should document --save");
    assert!(stdout.contains("--compare"), "Should document --compare");
    assert!(stdout.contains("--group-by"), "Should document --group-by");
}
