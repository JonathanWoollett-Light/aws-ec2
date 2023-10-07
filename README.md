# aws-ec2

A tool to run commands on AWS EC2 instances.

### Installation

```
cargo install aws-ec2
```

### Rust example

If you wanted to test your code on `t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04 you could run:

```
AWS_ACCESS_KEY_ID=<your aws access key> AWS_SECRET_ACCESS_KEY=<your aws secret access key> AWS_DEFAULT_REGION=<your aws region> aws-ec2 --path <path to my code> --instances t2.medium,t4g.medium --amis ami-0eb260c4d5475b901,ami-0e3f80b3d2a794117 --command "curl https://sh.rustup.rs -sSf | sh -s -- -y && ./cargo/bin/cargo test"
```

### Default example

By default it boots 2 instances (`t2.medium` Ubuntu 22.04 and `t4g.medium` Ubuntu 22.04) and runs the command `cat /proc/cpuinfo && ls`. See below.

```
jonathan@jonathan-Latitude-9510:~/Projects/test-aws$ AWS_ACCESS_KEY_ID=<your aws access key> AWS_SECRET_ACCESS_KEY=<your aws secret access key> AWS_DEFAULT_REGION=<your aws region> aws-ec2
   Compiling test-aws v0.1.0 (/home/jonathan/Projects/test-aws)
    Finished dev [unoptimized + debuginfo] target(s) in 50.12s
     Running `target/debug/test-aws`
2023-10-07T14:22:27.855817Z  INFO test_aws: Parsing command line arguments
2023-10-07T14:22:27.857384Z  INFO test_aws: Loading aws config
2023-10-07T14:22:28.083404Z  INFO test_aws: Creating SSH key pair
2023-10-07T14:22:28.084795Z  INFO lazy_load_credentials: aws_credential_types::cache::lazy_caching: credentials cache miss occurred; added new AWS credentials (took Ok(194.206µs))
2023-10-07T14:22:28.368567Z  INFO test_aws: Creating security groups
2023-10-07T14:22:28.701188Z  INFO test_aws: Setting ingress security group rule
2023-10-07T14:22:28.952538Z  INFO test_aws: Launching instances
2023-10-07T14:22:28.952538Z  INFO test_aws: Launching instances
2023-10-07T14:22:29.953191Z  INFO test_aws: Waiting for instance to enter the `running` state
2023-10-07T14:22:29.953656Z  INFO test_aws: Waiting for instance to enter the `running` state
2023-10-07T14:22:34.166194Z  INFO test_aws: Getting running instance description
2023-10-07T14:22:35.286090Z  INFO test_aws: Sleeping for 20s.
2023-10-07T14:22:52.710908Z  INFO test_aws: Getting running instance description
2023-10-07T14:22:52.832502Z  INFO test_aws: Sleeping for 20s.
2023-10-07T14:22:55.286440Z  INFO test_aws: Connecting SSH
2023-10-07T14:22:55.286587Z  INFO test_aws: socket_address: 13.40.32.87:22
2023-10-07T14:22:55.314946Z  INFO test_aws: SSH handshake
2023-10-07T14:22:55.536503Z  INFO test_aws: SSH authorize
2023-10-07T14:22:55.598561Z  INFO test_aws: Compressing source
2023-10-07T14:22:55.658530Z  INFO test_aws: Copying source
2023-10-07T14:22:55.658655Z  INFO test_aws: scp send
2023-10-07T14:22:56.427376Z  INFO test_aws: Wait for scp end of file
2023-10-07T14:22:56.506994Z  INFO test_aws: Closing scp
2023-10-07T14:22:56.507219Z  INFO test_aws: Waiting close scp
2023-10-07T14:22:56.507268Z  INFO test_aws: Decompressing source
2023-10-07T14:22:56.507296Z  INFO test_aws: Opening channel
2023-10-07T14:22:56.586566Z  INFO test_aws: Running exec: "tar -xf archive.tar.gz"
2023-10-07T14:22:56.617590Z  INFO test_aws: stdout: ""
2023-10-07T14:22:56.617785Z  INFO test_aws: stderr: ""
2023-10-07T14:22:56.617828Z  INFO test_aws: Waiting on close
2023-10-07T14:22:56.618211Z  INFO test_aws: Opening channel
2023-10-07T14:22:56.717051Z  INFO test_aws: Running exec: "cat /proc/cpuinfo && ls"
2023-10-07T14:22:56.743709Z  INFO test_aws: stdout: "processor\t: 0\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nprocessor\t: 1\nBogoMIPS\t: 243.75\nFeatures\t: fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm lrcpc dcpop asimddp ssbs\nCPU implementer\t: 0x41\nCPU architecture: 8\nCPU variant\t: 0x3\nCPU part\t: 0xd0c\nCPU revision\t: 1\n\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T14:22:56.743945Z  INFO test_aws: stderr: ""
2023-10-07T14:22:56.743982Z  INFO test_aws: Waiting on close
2023-10-07T14:22:56.744345Z  INFO test_aws: Sleeping for 30s.
2023-10-07T14:23:12.832762Z  INFO test_aws: Connecting SSH
2023-10-07T14:23:12.832820Z  INFO test_aws: socket_address: 18.168.148.137:22
2023-10-07T14:23:12.857483Z  INFO test_aws: SSH handshake
2023-10-07T14:23:13.100784Z  INFO test_aws: SSH authorize
2023-10-07T14:23:13.187308Z  INFO test_aws: Copying source
2023-10-07T14:23:13.187379Z  INFO test_aws: scp send
2023-10-07T14:23:14.607463Z  INFO test_aws: Wait for scp end of file
2023-10-07T14:23:14.715829Z  INFO test_aws: Closing scp
2023-10-07T14:23:14.716108Z  INFO test_aws: Waiting close scp
2023-10-07T14:23:14.716172Z  INFO test_aws: Decompressing source
2023-10-07T14:23:14.716217Z  INFO test_aws: Opening channel
2023-10-07T14:23:14.915649Z  INFO test_aws: Running exec: "tar -xf archive.tar.gz"
2023-10-07T14:23:14.945421Z  INFO test_aws: stdout: ""
2023-10-07T14:23:14.945513Z  INFO test_aws: stderr: ""
2023-10-07T14:23:14.945546Z  INFO test_aws: Waiting on close
2023-10-07T14:23:14.947473Z  INFO test_aws: Opening channel
2023-10-07T14:23:15.127676Z  INFO test_aws: Running exec: "cat /proc/cpuinfo && ls"
2023-10-07T14:23:15.155043Z  INFO test_aws: stdout: "processor\t: 0\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 79\nmodel name\t: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz\nstepping\t: 1\nmicrocode\t: 0xb000040\ncpu MHz\t\t: 2300.124\ncache size\t: 46080 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 0\ncpu cores\t: 2\napicid\t\t: 0\ninitial apicid\t: 0\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4600.00\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nprocessor\t: 1\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 79\nmodel name\t: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz\nstepping\t: 1\nmicrocode\t: 0xb000040\ncpu MHz\t\t: 2300.124\ncache size\t: 46080 KB\nphysical id\t: 0\nsiblings\t: 2\ncore id\t\t: 1\ncpu cores\t: 2\napicid\t\t: 2\ninitial apicid\t: 2\nfpu\t\t: yes\nfpu_exception\t: yes\ncpuid level\t: 13\nwp\t\t: yes\nflags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx rdtscp lm constant_tsc rep_good nopl xtopology cpuid tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm cpuid_fault invpcid_single pti fsgsbase bmi1 avx2 smep bmi2 erms invpcid xsaveopt\nbugs\t\t: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs itlb_multihit mmio_stale_data\nbogomips\t: 4600.00\nclflush size\t: 64\ncache_alignment\t: 64\naddress sizes\t: 46 bits physical, 48 bits virtual\npower management:\n\nCargo.lock\nCargo.toml\nREADME.md\narchive.tar.gz\nsrc\n"
2023-10-07T14:23:15.155398Z  INFO test_aws: stderr: ""
2023-10-07T14:23:15.155428Z  INFO test_aws: Waiting on close
2023-10-07T14:23:15.155634Z  INFO test_aws: Sleeping for 30s.
2023-10-07T14:23:26.744590Z  INFO test_aws: Terminate instances
2023-10-07T14:23:45.155814Z  INFO test_aws: Terminate instances
2023-10-07T14:23:45.703404Z  INFO test_aws: Deleting key pair
2023-10-07T14:23:45.851353Z  INFO test_aws: Sleeping for 120s.
2023-10-07T14:25:45.851588Z  INFO test_aws: Deleting security group
results: [Some(0), Some(0)]
jonathan@jonathan-Latitude-9510:~/Projects/test-aws$ 
```