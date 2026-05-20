use std::process::Command;

#[test]
fn variable_supply_example_prints_authority_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_variable-supply"))
        .output()
        .expect("run variable-supply example");

    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");

    assert!(stdout.contains("LP-0013 variable supply authority example"));
    assert!(stdout.contains("created variable mint decimals=6"));
    assert!(stdout.contains("minted initial supply=100"));
    assert!(stdout.contains("old authority rejected"));
    assert!(stdout.contains("new authority minted supply=125 balance=125"));
    assert!(stdout.contains("authority revoked"));
    assert!(stdout.contains("post-revoke mint rejected"));
}
