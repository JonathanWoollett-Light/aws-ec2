use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_aws-ec2");

#[test]
fn hello_world() {
    // Add `a` at front as it cannot start with a digit.
    let project_path = format!("/tmp/a{}", uuid::Uuid::new_v4());
    let output = Command::new("cargo")
        .args(["new", &project_path])
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 0);

    let output = Command::new(BINARY)
        .args([
            "--path",
            &project_path,
            "--command",
            "curl https://sh.rustup.rs -sSf | sh -s -- -y && \
            sudo apt-get -y update && \
            sudo apt -y install build-essential && \
            $HOME/.cargo/bin/cargo run",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 0);

    std::fs::remove_dir_all(project_path).unwrap();
}
