use std::process::Command;

#[test]
fn test_passing_workspace() {
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-workspace-lints"))
        .arg("workspace-lints")
        .arg("tests/passing-workspace/Cargo.toml")
        .output()
        .expect("Failed to run program");
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn test_failing_workspace() {
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-workspace-lints"))
        .arg("workspace-lints")
        .arg("tests/failing-workspace/Cargo.toml")
        .output()
        .expect("Failed to run program");
    assert!(!output.status.success(), "{output:?}");
    // Check that it mentions the crate that fails
    assert!(String::from_utf8_lossy(&output.stderr).contains("tests/failing-workspace/test-crate"));
}
