use std::process::Command;

#[test]
fn config_pda_gated_example_prints_gate_decision_flow() {
    let output = Command::new(env!("CARGO_BIN_EXE_config-pda-gated"))
        .output()
        .expect("run config-pda-gated example");

    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");

    assert!(stdout.contains("LP-0013 RFP-001 config PDA gated authority example"));
    assert!(stdout.contains("config pda gate derived"));
    assert!(stdout.contains("unauthorized config rejected"));
    assert!(stdout.contains("authorized config minted supply=64 balance=64"));
    assert!(stdout.contains("gate revoked"));
    assert!(stdout.contains("post-revoke config mint rejected"));
}
