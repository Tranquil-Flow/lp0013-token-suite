use std::process::Command;

#[test]
fn fixed_supply_example_prints_revoked_authority_behavior() {
    let output = Command::new(env!("CARGO_BIN_EXE_fixed-supply"))
        .output()
        .expect("run fixed-supply example");

    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");

    assert!(stdout.contains("LP-0013 fixed supply authority example"));
    assert!(stdout.contains("created fixed mint decimals=6"));
    assert!(stdout.contains("current authority=None"));
    assert!(stdout.contains("mint rejected because authority is revoked"));
}
