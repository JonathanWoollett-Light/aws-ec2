use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_aws-ec2");

#[test]
fn hello_world() {
    const COMMAND: &str = "\
        echo \"debconf debconf/frontend select Noninteractive\" | sudo debconf-set-selections \
        && sudo apt-get -y update \
        && sudo apt-get -y install build-essential \
        && curl https://sh.rustup.rs -sSf | sh -s -- -y \
        && $HOME/.cargo/bin/cargo new hello-world \
        && cd hello-world \
        && $HOME/.cargo/bin/cargo run \
    ";
    println!("command: {COMMAND}");

    let mut child = Command::new(BINARY)
        .args([
            "--instance",
            "t2.medium",
            "--ami",
            "ami-0eb260c4d5475b901",
            "--command",
            COMMAND,
        ])
        .spawn()
        .unwrap();

    let code = child.wait().unwrap();
    println!("code: {code}");
    // let output = Command::new(BINARY)
    //     .args([
    //         "--instance",
    //         "t2.medium",
    //         "--ami",
    //         "ami-0eb260c4d5475b901",
    //         "--command",
    //         COMMAND,
    //     ])
    //     .output()
    //     .unwrap();
    // println!("stderr: {}",std::str::from_utf8(&output.stderr).unwrap());
    // println!("stdout: {}",std::str::from_utf8(&output.stdout).unwrap());
    // assert_eq!(output.status.code().unwrap(), 0);

    // std::fs::remove_dir_all(project_path).unwrap();
}
