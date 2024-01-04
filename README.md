# aws-ec2

[![Crates.io](https://img.shields.io/crates/v/aws-ec2)](https://crates.io/crates/aws-ec2)

A tool to run commands on AWS EC2 instances.

### Purpose

When developing real world software we often want to test our code across many specific enviroments; often the enviroments in which it will be deployed, which will often be AWS EC2.

This can either require services like [buildjet](https://buildjet.com/for-github-actions) or more complex infrastructure setups that can absorb developer time.

This project was sparked from my work on [firecracker](https://github.com/firecracker-microvm/firecracker) and [nix](https://github.com/nix-rust/nix) and thinking about how to simplify their respective CIs and maintain the same level of platform coverage.

Unlike the aforementioned solutions this can run anywhere, without any setup*. A contributor can test the code themselves without requiring maintainer intervention.

*Some things like AWS EC2 vcpu limits are unfortunately unavoiable, so in applications like [firecracker](https://github.com/firecracker-microvm/firecracker) which launch multiple very big `.metal` instances it would require an AWS account with specific settings to support this.

### Overview

It creates its own resources and cleans up after itself.

The process approximately follows:

1. Creates key pair.
2. Creates security group.
3. If `--path` is given compresses directory into a `.tar.gz` archive.
4. Start the instance.
5. If `--path` is given copy across the `.tar.gz` archive.
6. If `--path` is given decompress `.tar.gz` archive.
7. Run `--command`.
8. Terminate instance.
9. Delete key pair.
10. Delete security group.

### Installation

```
cargo +nightly install aws-ec2
```


### Examples

#### Default

By default it runs the command `cat /proc/cpuinfo && uname -a && ls`.

```
AWS_ACCESS_KEY_ID=<public key> \
AWS_SECRET_ACCESS_KEY=<private key> \
AWS_DEFAULT_REGION=eu-west-2 \
aws-ec2 --instance t2.medium --ami ami-0eb260c4d5475b901
```

#### Rust Hello World!

Running `Hello World!` on `t2.medium` Ubuntu 22.04 with 32gb EBS volumes you could run:

```
AWS_ACCESS_KEY_ID=<public key> \
AWS_SECRET_ACCESS_KEY=<private key> \
AWS_DEFAULT_REGION=eu-west-2 \
aws-ec2 \
--size 32 \
--instance t2.medium \
--ami ami-0eb260c4d5475b901 \
--command "\
    echo \"debconf debconf/frontend select Noninteractive\" | sudo debconf-set-selections \
    && sudo apt-get -y update \
    && sudo apt-get -y install build-essential \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && \$HOME/.cargo/bin/cargo new hello-world \
    && cd hello-world \
    && \$HOME/.cargo/bin/cargo run \
"
```

#### Testing a Rust project

If you wanted to test your code on `t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04 with 32gb EBS volumes you could run:

```
AWS_ACCESS_KEY_ID=<public key> \
AWS_SECRET_ACCESS_KEY=<private key> \
AWS_DEFAULT_REGION=eu-west-2 \
aws-ec2 \
--size 32 \
--instance t2.medium \
--ami ami-0eb260c4d5475b901 \
--command "\
    echo \"debconf debconf/frontend select Noninteractive\" | sudo debconf-set-selections \
    && sudo apt-get -y update \
    && sudo apt-get -y install git build-essential \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && git clone --depth 1 --branch <branch> <repo> <directory> \
    && cd <directory> \
    && \$HOME/.cargo/bin/cargo test \
"
```

if you want to use your local files you could run:

```
AWS_ACCESS_KEY_ID=<public key> \
AWS_SECRET_ACCESS_KEY=<private key> \
AWS_DEFAULT_REGION=eu-west-2 \
aws-ec2 \
--size 32 \
--instance t2.medium,t4g.medium \
--ami ami-0eb260c4d5475b901,ami-0e3f80b3d2a794117 \
--path <path to your project>
--command "\
    echo \"debconf debconf/frontend select Noninteractive\" | sudo debconf-set-selections \
    && sudo apt-get -y update \
    && sudo apt-get -y install build-essential \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && \$HOME/.cargo/bin/cargo test \
"
```