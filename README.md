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
--command "curl https://sh.rustup.rs -sSf | sh -s -- -y && ./cargo/bin/cargo test"
```

### Default example

By default it boots 2 instances (`t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04) and runs the command `cat /proc/cpuinfo && ls`.

```
jonathan@jonathan-Latitude-9510:~/Projects/aws-ec2$ AWS_ACCESS_KEY_ID=<public key> AWS_SECRET_ACCESS_KEY=<secret key> AWS_DEFAULT_REGION=eu-west-2 cargo run
   Compiling aws-ec2 v0.1.0 (/home/jonathan/Projects/aws-ec2)
    Finished dev [unoptimized + debuginfo] target(s) in 16.65s
     Running `target/debug/aws-ec2`
2023-10-07T19:50:47.097809Z  INFO aws_ec2: Parsing command line arguments
2023-10-07T19:50:47.098164Z  INFO aws_ec2: Loading aws config
2023-10-07T19:50:47.153656Z  INFO aws_ec2: Creating SSH key pair
2023-10-07T19:50:47.154197Z  INFO lazy_load_credentials: aws_credential_types::cache::lazy_caching: credentials cache miss occurred; added new AWS credentials (took Ok(62.258µs))
2023-10-07T19:50:47.537317Z  INFO aws_ec2: Creating security groups
2023-10-07T19:50:47.848167Z  INFO aws_ec2: Setting ingress security group rule
2023-10-07T19:50:48.155623Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Launching instances
2023-10-07T19:50:48.155623Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Launching instances
2023-10-07T19:50:49.176923Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting for instance to enter the `running` state
2023-10-07T19:50:49.176924Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting for instance to enter the `running` state
2023-10-07T19:50:53.306254Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Getting running instance description
2023-10-07T19:50:54.410354Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Sleeping for 20s.
2023-10-07T19:51:14.410589Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Connecting SSH
2023-10-07T19:51:14.410666Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: socket_address: 3.8.122.23:22
2023-10-07T19:51:14.436750Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: SSH handshake
2023-10-07T19:51:14.665069Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: SSH authorize
2023-10-07T19:51:14.735423Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Compressing source
2023-10-07T19:51:14.773455Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Copying source
2023-10-07T19:51:14.773512Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: scp send
2023-10-07T19:51:15.805600Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Wait for scp end of file
2023-10-07T19:51:15.893666Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Closing scp
2023-10-07T19:51:15.893746Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting close scp
2023-10-07T19:51:15.893765Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Decompressing source
2023-10-07T19:51:15.893780Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Opening channel
2023-10-07T19:51:15.984777Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Running exec: "tar -xf archive.tar.gz"
2023-10-07T19:51:16.015621Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stdout: ""
2023-10-07T19:51:16.015675Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stderr: ""
2023-10-07T19:51:16.015687Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting on close
2023-10-07T19:51:16.015741Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Opening channel
2023-10-07T19:51:16.105472Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Running exec: "cat /proc/cpuinfo && ls"
2023-10-07T19:51:16.133633Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stdout: "processor\t: 0\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nprocessor\t: 1\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T19:51:16.133748Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: stderr: ""
2023-10-07T19:51:16.133783Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Waiting on close
2023-10-07T19:51:16.133946Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Sleeping for 30s.
2023-10-07T19:51:18.311785Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Getting running instance description
2023-10-07T19:51:18.479485Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Sleeping for 20s.
2023-10-07T19:51:38.479853Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Connecting SSH
2023-10-07T19:51:38.480027Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: socket_address: 13.40.215.252:22
2023-10-07T19:51:38.506367Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: SSH handshake
2023-10-07T19:51:38.755971Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: SSH authorize
2023-10-07T19:51:38.825992Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Copying source
2023-10-07T19:51:38.826046Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: scp send
2023-10-07T19:51:39.796159Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Wait for scp end of file
2023-10-07T19:51:39.877382Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Closing scp
2023-10-07T19:51:39.877478Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting close scp
2023-10-07T19:51:39.877502Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Decompressing source
2023-10-07T19:51:39.877522Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Opening channel
2023-10-07T19:51:39.972538Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Running exec: "tar -xf archive.tar.gz"
2023-10-07T19:51:40.006119Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stdout: ""
2023-10-07T19:51:40.006179Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stderr: ""
2023-10-07T19:51:40.006191Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting on close
2023-10-07T19:51:40.006250Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Opening channel
2023-10-07T19:51:40.096242Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Running exec: "cat /proc/cpuinfo && ls"
2023-10-07T19:51:40.126819Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stdout: "processor\t: 0\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 79\nmodel name\t: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz\nstepping\t: 1\nmicrocode\t: 0xb000040\ncpu MHz\t\t: 2299.886\ncache size\t: 46080 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 0\ncpu cores\t: 2\napicid\t\t: 0\ninitial apicid\t: 0\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4600.04\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nprocessor\t: 1\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 79\nmodel name\t: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz\nstepping\t: 1\nmicrocode\t: 0xb000040\ncpu MHz\t\t: 2299.886\ncache size\t: 46080 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 1\ncpu cores\t: 2\napicid\t\t: 2\ninitial apicid\t: 2\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4600.04\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T19:51:40.126971Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: stderr: ""
2023-10-07T19:51:40.126983Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Waiting on close
2023-10-07T19:51:40.127093Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Sleeping for 30s.
2023-10-07T19:51:46.134056Z  INFO run_instance{instance_type=T4gMedium ami="ami-0e3f80b3d2a794117"}: aws_ec2: Terminate instances
2023-10-07T19:52:10.127333Z  INFO run_instance{instance_type=T2Medium ami="ami-0eb260c4d5475b901"}: aws_ec2: Terminate instances
2023-10-07T19:52:10.510529Z  INFO aws_ec2: Deleting key pair
2023-10-07T19:52:10.686370Z  INFO aws_ec2: Sleeping for 120s.
2023-10-07T19:54:10.686540Z  INFO aws_ec2: Deleting security group
results: [Some(0), Some(0)]
jonathan@jonathan-Latitude-9510:~/Projects/aws-ec2$ 
```
