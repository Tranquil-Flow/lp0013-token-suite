use std::process::Command;

#[test]
fn demo_variable_lifecycle_prints_authority_flow() {
    let output = Command::new(env!("CARGO_BIN_EXE_mint-cli"))
        .arg("demo-variable")
        .output()
        .expect("run mint-cli demo-variable");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("created variable mint decimals=6"));
    assert!(stdout.contains("minted amount=100 supply=100 balance=100"));
    assert!(stdout.contains("rotated authority"));
    assert!(stdout.contains("old authority rejected"));
    assert!(stdout.contains("minted amount=25 supply=125 balance=125"));
    assert!(stdout.contains("revoked authority"));
    assert!(stdout.contains("post-revoke mint rejected"));
}

#[test]
fn demo_fixed_lifecycle_prints_revoked_flow() {
    let output = Command::new(env!("CARGO_BIN_EXE_mint-cli"))
        .arg("demo-fixed")
        .output()
        .expect("run mint-cli demo-fixed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("created fixed mint decimals=6"));
    assert!(stdout.contains("mint rejected: authority has been revoked"));
}
