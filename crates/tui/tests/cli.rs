use std::{fs, process::Command};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn headless_snapshot_and_cli_failures_are_actionable() -> TestResult {
    let root = std::env::temp_dir().join(format!("square-cli-test-{}", std::process::id()));
    fs::create_dir_all(&root)?;
    let snapshot = root.join("screen.txt");
    let good = Command::new(env!("CARGO_BIN_EXE_tui"))
        .current_dir(&root)
        .args(["demo", "--at", "167", "--snapshot"])
        .arg(&snapshot)
        .output()?;
    assert!(
        good.status.success(),
        "{}",
        String::from_utf8_lossy(&good.stderr)
    );
    assert!(fs::read_to_string(&snapshot)?.contains("DEMO"));
    let cases: &[(&[&str], &str)] = &[
        (&["demo", "--at", "601"], "exceeds fixture duration"),
        (&["demo"], "interactive terminal required"),
        (&["demo", "--fixture", "absent.json"], "cannot read fixture"),
        (
            &["demo", "--snapshot", "missing/preview.txt"],
            "cannot write snapshot",
        ),
        (&["demo", "--width", "0"], "invalid value"),
    ];
    for (args, message) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_tui"))
            .current_dir(&root)
            .args(*args)
            .output()?;
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{args:?}"
        );
    }
    fs::remove_dir_all(root)?;
    Ok(())
}
