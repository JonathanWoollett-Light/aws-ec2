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
3. Compresses `--path` into a `.tar.gz` archive.
4. For each pair from `--instances` and `--amis`:
   1. Start the instance.
   2. Copy across the `.tar.gz` archive.
   3. Decompress `.tar.gz` archive.
   4. Run `--command`.
   5. Terminate instance.
5. Delete key pair.
6. Delete security group.
7. Return exit code (0 if all commands return 0, else 1).

### Installation

```
cargo +nightly install aws-ec2
```

### Rust example

If you wanted to test your code on `t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04 with 32gb EBS volumes you could run:

```
AWS_ACCESS_KEY_ID=<public key> \
AWS_SECRET_ACCESS_KEY=<secret key> \
AWS_DEFAULT_REGION=eu-west-2 \
aws-ec2 \
--path <path to your code> \
--size 32 \
--instances t2.medium,t4g.medium \
--amis ami-0eb260c4d5475b901,ami-0e3f80b3d2a794117 \
--command "curl https://sh.rustup.rs -sSf | sh -s -- -y && sudo apt-get -y update && sudo apt -y install build-essential && $HOME/.cargo/bin/cargo test"
```

### Default example

By default it boots 2 instances (`t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04) and runs the command `cat /proc/cpuinfo && uname -a && ls`.

```
jonathan@jonathan-Latitude-9510:~/Projects/aws-ec2$ AWS_ACCESS_KEY_ID=<public key> AWS_SECRET_ACCESS_KEY=<secret key> AWS_DEFAULT_REGION=eu-west-2 cargo run --path ./
   Compiling aws-ec2 v0.1.0 (/home/jonathan/Projects/aws-ec2)
    Finished dev [unoptimized + debuginfo] target(s) in 18.82s
     Running `target/debug/aws-ec2`
2023-10-07T20:10:34.780847Z  INFO aws_ec2: Parsing command line arguments
2023-10-07T20:10:34.781480Z  INFO aws_ec2: Loading aws config
2023-10-07T20:10:34.870813Z  INFO aws_ec2: Creating SSH key pair
2023-10-07T20:10:34.871546Z  INFO lazy_load_credentials: aws_credential_types::cache::lazy_caching: credentials cache miss occurred; added new AWS credentials (took Ok(89.987µs))
2023-10-07T20:10:35.286662Z  INFO aws_ec2: Creating security groups
2023-10-07T20:10:35.532858Z  INFO aws_ec2: Setting ingress security group rule
2023-10-07T20:10:35.915736Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Launching instances
2023-10-07T20:10:35.915737Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Launching instances
2023-10-07T20:10:36.822949Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting for instance to enter the `running` state
2023-10-07T20:10:37.828169Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting for instance to enter the `running` state
2023-10-07T20:10:41.838106Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Getting running instance description
2023-10-07T20:10:42.948820Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Sleeping for 20s.
2023-10-07T20:10:58.142456Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Getting running instance description
2023-10-07T20:10:58.254070Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Sleeping for 20s.
2023-10-07T20:11:02.949014Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Connecting SSH
2023-10-07T20:11:02.949082Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: socket_address: 18.130.253.74:22
2023-10-07T20:11:02.973981Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: SSH handshake
2023-10-07T20:11:03.191838Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: SSH authorize
2023-10-07T20:11:03.262781Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Compressing source
2023-10-07T20:11:03.316414Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Copying source
2023-10-07T20:11:03.316492Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: scp send
2023-10-07T20:11:04.175898Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Wait for scp end of file
2023-10-07T20:11:04.275208Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Closing scp
2023-10-07T20:11:04.275370Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting close scp
2023-10-07T20:11:04.275415Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Decompressing source
2023-10-07T20:11:04.275446Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Opening channel
2023-10-07T20:11:04.382579Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Running exec: "tar -xf /tmp/fd904d93-1bea-42ad-9859-aa5af70f350a"
2023-10-07T20:11:04.412740Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stdout: ""
2023-10-07T20:11:04.412824Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stderr: ""
2023-10-07T20:11:04.412847Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting on close
2023-10-07T20:11:04.412947Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Opening channel
2023-10-07T20:11:04.503819Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Running exec: "cat /proc/cpuinfo && uname -a && ls"
2023-10-07T20:11:04.528576Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stdout: "processor\t: 0\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nprocessor\t: 1\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nLinux ip-172-31-25-41 5.19.0-1025-aws #26~22.04.1-Ubuntu SMP Mon Apr 24 01:58:03 UTC 2023 aarch64 aarch64 aarch64 GNU/Linux\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T20:11:04.528720Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stderr: ""
2023-10-07T20:11:04.528759Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting on close
2023-10-07T20:11:04.528890Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Sleeping for 30s.
2023-10-07T20:11:18.254245Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Connecting SSH
2023-10-07T20:11:18.254385Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: socket_address: 13.40.189.230:22
2023-10-07T20:11:18.283070Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: SSH handshake
2023-10-07T20:11:18.512180Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: SSH authorize
2023-10-07T20:11:18.582919Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Copying source
2023-10-07T20:11:18.582995Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: scp send
2023-10-07T20:11:19.483127Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Wait for scp end of file
2023-10-07T20:11:19.579623Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Closing scp
2023-10-07T20:11:19.579714Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting close scp
2023-10-07T20:11:19.579737Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Decompressing source
2023-10-07T20:11:19.579754Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Opening channel
2023-10-07T20:11:19.663290Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Running exec: "tar -xf /tmp/3c3cd039-f6ee-42eb-ae94-196b3c3baff4"
2023-10-07T20:11:19.694849Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stdout: ""
2023-10-07T20:11:19.694931Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stderr: ""
2023-10-07T20:11:19.694955Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting on close
2023-10-07T20:11:19.696865Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Opening channel
2023-10-07T20:11:19.802893Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Running exec: "cat /proc/cpuinfo && uname -a && ls"
2023-10-07T20:11:19.832729Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stdout: "processor\t: 0\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 63\nmodel name\t: Intel(R) Xeon(R) CPU E5-2676 v3 @ 2.40GHz\nstepping\t: 2\nmicrocode\t: 0x49\ncpu MHz\t\t: 2399.896\ncache size\t: 30720 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 0\ncpu cores\t: 2\napicid\t\t: 0\ninitial apicid\t: 0\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4800.01\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nprocessor\t: 1\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 63\nmodel name\t: Intel(R) Xeon(R) CPU E5-2676 v3 @ 2.40GHz\nstepping\t: 2\nmicrocode\t: 0x49\ncpu MHz\t\t: 2399.896\ncache size\t: 30720 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 1\ncpu cores\t: 2\napicid\t\t: 2\ninitial apicid\t: 2\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4800.01\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nLinux ip-172-31-43-49 5.19.0-1025-aws #26~22.04.1-Ubuntu SMP Mon Apr 24 01:58:15 UTC 2023 x86_64 x86_64 x86_64 GNU/Linux\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T20:11:19.832854Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stderr: ""
2023-10-07T20:11:19.832870Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting on close
2023-10-07T20:11:19.832936Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Sleeping for 30s.
2023-10-07T20:11:34.529150Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Terminate instances
2023-10-07T20:11:49.833085Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Terminate instances
2023-10-07T20:11:50.205850Z  INFO aws_ec2: Deleting key pair
2023-10-07T20:11:50.366317Z  INFO aws_ec2: Sleeping for 120s.
2023-10-07T20:13:50.366632Z  INFO aws_ec2: Deleting security group
results: [Some(0), Some(0)]
jonathan@jonathan-Latitude-9510:~/Projects/aws-ec2$ 
```
