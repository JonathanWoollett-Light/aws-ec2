# aws-ec2

[![Crates.io](https://img.shields.io/crates/v/aws-ec2)](https://crates.io/crates/aws-ec2)

A tool to run commands on AWS EC2 instances.

### Purpose

When developing real world software we often want to test our code across many specific enviroments; often the enviroments in which it will be deployed, which will often be AWS EC2.

This can either require services like [buildjet](https://buildjet.com/for-github-actions) or more complex infrastructure setups that can absorb developer time.

This project was sparked from my work on [firecracker](https://github.com/firecracker-microvm/firecracker) and [nix](https://github.com/nix-rust/nix) and thinking about how to simplify their respective CIs and maintain the same level of platform coverage.

### Overview

It creates its own resources and cleans up after itself.

The process approximately follows:

1. Creates key pair.
2. Creates security group.
3. Compresses `--path` into `archive.tar.gz`.
4. For each pair from `--instances` and `--amis`:
   1. Start the instance.
   2. Copy across `archive.tar.gz`
   3. Decompress `archive.tar.gz`
   4. Run `--command`
 5. Terminate instance
5. Delete key pair.
6. Delete security group.
7. Return exit code (0 if all commands return 0, else 1).

### Installation

```
cargo install aws-ec2
```

### Rust example

If you wanted to test your code on `t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04 you could run:

```
AWS_ACCESS_KEY_ID=<public key> AWS_SECRET_ACCESS_KEY=<secret key> AWS_DEFAULT_REGION=eu-west-2 aws-ec2 --path <path to your code> --instances t2.medium,t4g.medium --amis ami-0eb260c4d5475b901,ami-0e3f80b3d2a794117 --command "curl https://sh.rustup.rs -sSf | sh -s -- -y && ./cargo/bin/cargo test"
```

### Default example

By default it boots 2 instances (`t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04) and runs the command `cat /proc/cpuinfo && ls`.

```
jonathan@jonathan-Latitude-9510:~/Projects/aws-ec2$ AWS_ACCESS_KEY_ID=<public key> AWS_SECRET_ACCESS_KEY=<secret key> AWS_DEFAULT_REGION=eu-west-2 cargo run
   Compiling aws-ec2 v0.1.0 (/home/jonathan/Projects/aws-ec2)
    Finished dev [unoptimized + debuginfo] target(s) in 17.69s
     Running `target/debug/aws-ec2`
2023-10-07T18:46:17.705956Z  INFO ThreadId(01) aws_ec2: Parsing command line arguments
2023-10-07T18:46:17.706557Z  INFO ThreadId(01) aws_ec2: Loading aws config
2023-10-07T18:46:17.819462Z  INFO ThreadId(01) aws_ec2: Creating SSH key pair
2023-10-07T18:46:17.820439Z  INFO ThreadId(01) lazy_load_credentials: aws_credential_types::cache::lazy_caching: credentials cache miss occurred; added new AWS credentials (took Ok(122.285µs))
2023-10-07T18:46:18.066580Z  INFO ThreadId(01) aws_ec2: Creating security groups
2023-10-07T18:46:18.301684Z  INFO ThreadId(01) aws_ec2: Setting ingress security group rule
2023-10-07T18:46:18.598571Z  INFO ThreadId(02) aws_ec2: Launching instances
2023-10-07T18:46:18.598593Z  INFO ThreadId(12) aws_ec2: Launching instances
2023-10-07T18:46:19.444878Z  INFO ThreadId(11) aws_ec2: Waiting for instance to enter the `running` state
2023-10-07T18:46:20.448672Z  INFO ThreadId(11) aws_ec2: Waiting for instance to enter the `running` state
2023-10-07T18:46:24.460993Z  INFO ThreadId(11) aws_ec2: Getting running instance description
2023-10-07T18:46:25.577065Z  INFO ThreadId(02) aws_ec2: Sleeping for 20s.
2023-10-07T18:46:45.577297Z  INFO ThreadId(02) aws_ec2: Connecting SSH
2023-10-07T18:46:45.577389Z  INFO ThreadId(02) aws_ec2: socket_address: 18.133.233.156:22
2023-10-07T18:46:45.616239Z  INFO ThreadId(02) aws_ec2: SSH handshake
2023-10-07T18:46:45.862444Z  INFO ThreadId(02) aws_ec2: SSH authorize
2023-10-07T18:46:45.932438Z  INFO ThreadId(02) aws_ec2: Compressing source
2023-10-07T18:46:45.955183Z  INFO ThreadId(02) aws_ec2: Copying source
2023-10-07T18:46:45.955216Z  INFO ThreadId(02) aws_ec2: scp send
2023-10-07T18:46:46.892681Z  INFO ThreadId(02) aws_ec2: Wait for scp end of file
2023-10-07T18:46:46.972397Z  INFO ThreadId(02) aws_ec2: Closing scp
2023-10-07T18:46:46.972472Z  INFO ThreadId(02) aws_ec2: Waiting close scp
2023-10-07T18:46:46.972488Z  INFO ThreadId(02) aws_ec2: Decompressing source
2023-10-07T18:46:46.972499Z  INFO ThreadId(02) aws_ec2: Opening channel
2023-10-07T18:46:47.075241Z  INFO ThreadId(02) aws_ec2: Running exec: "tar -xf archive.tar.gz"
2023-10-07T18:46:47.112561Z  INFO ThreadId(02) aws_ec2: stdout: ""
2023-10-07T18:46:47.112598Z  INFO ThreadId(02) aws_ec2: stderr: ""
2023-10-07T18:46:47.112606Z  INFO ThreadId(02) aws_ec2: Waiting on close
2023-10-07T18:46:47.112667Z  INFO ThreadId(02) aws_ec2: Opening channel
2023-10-07T18:46:47.203685Z  INFO ThreadId(02) aws_ec2: Running exec: "cat /proc/cpuinfo && ls"
2023-10-07T18:46:47.231916Z  INFO ThreadId(02) aws_ec2: stdout: "processor\t: 0\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nprocessor\t: 1\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T18:46:47.231966Z  INFO ThreadId(02) aws_ec2: stderr: ""
2023-10-07T18:46:47.231974Z  INFO ThreadId(02) aws_ec2: Waiting on close
2023-10-07T18:46:47.232032Z  INFO ThreadId(02) aws_ec2: Sleeping for 30s.
2023-10-07T18:46:50.437304Z  INFO ThreadId(11) aws_ec2: Getting running instance description
2023-10-07T18:46:50.543345Z  INFO ThreadId(10) aws_ec2: Sleeping for 20s.
2023-10-07T18:47:10.543586Z  INFO ThreadId(10) aws_ec2: Connecting SSH
2023-10-07T18:47:10.543678Z  INFO ThreadId(10) aws_ec2: socket_address: 18.170.215.138:22
2023-10-07T18:47:10.569130Z  INFO ThreadId(10) aws_ec2: SSH handshake
2023-10-07T18:47:10.823027Z  INFO ThreadId(10) aws_ec2: SSH authorize
2023-10-07T18:47:10.902977Z  INFO ThreadId(10) aws_ec2: Copying source
2023-10-07T18:47:10.903014Z  INFO ThreadId(10) aws_ec2: scp send
2023-10-07T18:47:11.873167Z  INFO ThreadId(10) aws_ec2: Wait for scp end of file
2023-10-07T18:47:11.962938Z  INFO ThreadId(10) aws_ec2: Closing scp
2023-10-07T18:47:11.963011Z  INFO ThreadId(10) aws_ec2: Waiting close scp
2023-10-07T18:47:11.963024Z  INFO ThreadId(10) aws_ec2: Decompressing source
2023-10-07T18:47:11.963033Z  INFO ThreadId(10) aws_ec2: Opening channel
2023-10-07T18:47:12.052241Z  INFO ThreadId(10) aws_ec2: Running exec: "tar -xf archive.tar.gz"
2023-10-07T18:47:12.093042Z  INFO ThreadId(10) aws_ec2: stdout: ""
2023-10-07T18:47:12.093076Z  INFO ThreadId(10) aws_ec2: stderr: ""
2023-10-07T18:47:12.093083Z  INFO ThreadId(10) aws_ec2: Waiting on close
2023-10-07T18:47:12.093139Z  INFO ThreadId(10) aws_ec2: Opening channel
2023-10-07T18:47:12.194343Z  INFO ThreadId(10) aws_ec2: Running exec: "cat /proc/cpuinfo && ls"
2023-10-07T18:47:12.222515Z  INFO ThreadId(10) aws_ec2: stdout: "processor\t: 0\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 79\nmodel name\t: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz\nstepping\t: 1\nmicrocode\t: 0xb000040\ncpu MHz\t\t: 2299.936\ncache size\t: 46080 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 0\ncpu cores\t: 2\napicid\t\t: 0\ninitial apicid\t: 0\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4600.01\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nprocessor\t: 1\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 79\nmodel name\t: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz\nstepping\t: 1\nmicrocode\t: 0xb000040\ncpu MHz\t\t: 2299.936\ncache size\t: 46080 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 1\ncpu cores\t: 2\napicid\t\t: 2\ninitial apicid\t: 2\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4600.01\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T18:47:12.222600Z  INFO ThreadId(10) aws_ec2: stderr: ""
2023-10-07T18:47:12.222607Z  INFO ThreadId(10) aws_ec2: Waiting on close
2023-10-07T18:47:12.222657Z  INFO ThreadId(10) aws_ec2: Sleeping for 30s.
2023-10-07T18:47:17.232147Z  INFO ThreadId(02) aws_ec2: Terminate instances
2023-10-07T18:47:42.222803Z  INFO ThreadId(10) aws_ec2: Terminate instances
2023-10-07T18:47:42.669018Z  INFO ThreadId(01) aws_ec2: Deleting key pair
2023-10-07T18:47:42.873911Z  INFO ThreadId(01) aws_ec2: Sleeping for 120s.
2023-10-07T18:49:42.874045Z  INFO ThreadId(01) aws_ec2: Deleting security group
results: [Some(0), Some(0)]
jonathan@jonathan-Latitude-9510:~/Projects/aws-ec2$ 
```
