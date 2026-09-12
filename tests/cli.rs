use std::process::Command;

const TERMINAL_SEQUENCES: [&str; 4] = ["\x1b[?1049h", "\x1b[?1049l", "\x1b[?1000h", "\x1b[?1000l"];

fn assert_terminal_was_not_modified(output: &[u8]) {
    let output = String::from_utf8_lossy(output);
    for sequence in TERMINAL_SEQUENCES {
        assert!(!output.contains(sequence));
    }
}

#[test]
fn help_exits_before_terminal_setup() {
    let output = Command::new(env!("CARGO_BIN_EXE_stranger"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: stranger"));
    assert!(output.stderr.is_empty());
    assert_terminal_was_not_modified(&output.stdout);
    assert_terminal_was_not_modified(&output.stderr);
}

#[test]
fn invalid_arguments_exit_before_terminal_setup() {
    let output = Command::new(env!("CARGO_BIN_EXE_stranger"))
        .arg("--definitely-invalid")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
    assert_terminal_was_not_modified(&output.stdout);
    assert_terminal_was_not_modified(&output.stderr);
}
