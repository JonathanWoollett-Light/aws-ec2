#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(async_closure)]
#![warn(clippy::pedantic)]
#![allow(clippy::type_complexity)]

use aws_sdk_ec2 as ec2;
use clap::Parser;
use ec2::types::InstanceType;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::ErrorKind;

use std::io::ErrorKind::WouldBlock;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use std::process::ExitCode;
use std::str::FromStr;

use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use tracing::info;

/// The default port used by ec2 for ssh.
const EC2_SSH_PORT: VolumeSize = 22;

// TODO This should only be default for optional command line argument.
const INSTANCE_POLL_STATE_SLEEP: Duration = Duration::from_secs(1);

// TODO This should only be default for optional command line argument.
const SECURITY_GROUP_DESCRIPTION: &str = "test-aws-security-group-description";

/// Default command to run on the host.
const DEFAULT_COMMAND: &str = "cat /proc/cpuinfo && uname -a && ls";

// TODO Remove this.
const RUN_BUFFER: Duration = Duration::from_secs(30);

// TODO Replace this with polling.
const SSH_STARTUP_BUFFER: Duration = Duration::from_secs(20);

// // TODO Replace this with polling so it doesn't need to wait longer than neccessary.
// /// We need to wait a long time for the instance to be terminated and for the security group to lose
// /// its dependency so it can be deleted.
// const DELETE_SECURITY_GROUP_BUFFER: Duration = Duration::from_secs(120);

const DEFAULT_COMMAND_TIMEOUT_SECS: u64 = 300;

const DEFAULT_SIZE: VolumeSize = 16;

type VolumeSize = u16;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    path: Option<String>,
    /// Name of the SSH key pair used.
    #[arg(long)]
    key_name: Option<String>,
    /// The name to use for the security group for instances.
    #[arg(long)]
    security_group_name: Option<String>,
    /// Timeout in seconds.
    #[arg(long)]
    timeout: Option<u64>,
    /// The command to run on the instance.
    #[arg(long)]
    command: Option<String>,
    /// The size in GB of each EBS volume to attach to each instance.
    #[arg(long)]
    size: Option<VolumeSize>,
    /// The EC2 instance type.
    #[arg(long)]
    instance: InstanceType,
    /// The EC2 AMI.
    #[arg(long)]
    ami: String,
}

type SdkResponse = http::response::Response<aws_smithy_http::body::SdkBody>;
type SdkError<E> = aws_smithy_http::result::SdkError<E, SdkResponse>;

#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error("Failed to create key pair: {0}")]
    CreateKeyPair(SdkError<aws_sdk_ec2::operation::create_key_pair::CreateKeyPairError>),
    #[error("Created key pair missing key material.")]
    CreateKeyPairMaterial,
    #[error("Failed to create security group: {0}")]
    CreateSecurityGroup(
        SdkError<aws_sdk_ec2::operation::create_security_group::CreateSecurityGroupError>,
    ),
    #[error("Created security group missing id.")]
    CreateSecurityGroupId,
    #[error("Failed to create security group ingress rule: {0}")]
    AuthorizeSecurityGroupIngress(SdkError<aws_sdk_ec2::operation::authorize_security_group_ingress::AuthorizeSecurityGroupIngressError>),
    #[error("Failed to run instances: {0}")]
    RunInstances(SdkError<aws_sdk_ec2::operation::run_instances::RunInstancesError>),
    #[error("Missing instance id from run instances.")]
    RunInstancesInstanceId,
    #[error("Instance failed to enter running state within timeout.")]
    StartupTimeout,
    #[error("Failed to describe instance status: {0}")]
    DescribeInstanceStatus(SdkError<aws_sdk_ec2::operation::describe_instance_status::DescribeInstanceStatusError>),
    #[error("Missing state from describe instance status.")]
    DescribeInstanceStatusState,
    #[error("Failed to describe instances: {0}")]
    DescribeInstances(SdkError<aws_sdk_ec2::operation::describe_instances::DescribeInstancesError>),
    #[error("Missing public ip address from describe instances.")]
    DescribeInstancesPublicIpAddress,
    #[error("Failed to parse public ip address: {0}")]
    PublicIpParse(std::net::AddrParseError),
    #[error("Failed to connect TCP stream: {0}")]
    TcpStreamConnect(std::io::Error),
    #[error("Failed to create SSH session: {0}")]
    SshSession(ssh2::Error),
    #[error("Failed SSH handshake: {0}")]
    SshHandshake(std::io::Error),
    #[error("Timed out attempting SSH handshake.")]
    SshHandshakeTimeout,
    #[error("Failed to setup SSH auth: {0}")]
    SshAuthSetup(std::io::Error),
    #[error("Failed SSH auth.")]
    SshAuthFailed,
    #[error("Failed to read directory: {0}")]
    ReadDir(std::io::Error),
    #[error("Failed to read entry: {0}")]
    ReadEntry(std::io::Error),
    #[error("Failed to read file type: {0}")]
    ReadFileType(std::io::Error),
    #[error("Failed to append directory to source archive: {0}")]
    AppendDir(std::io::Error),
    #[error("Failed to append file to source archive: {0}")]
    AppendFile(std::io::Error),
    #[error("Failed to complete archive: {0}")]
    CompleteArchive(std::io::Error),
    #[error("Failed to start scp: {0}")]
    ScpSend(std::io::Error),
    #[error("Failed to write to scp: {0}")]
    ScpWrite(std::io::Error),
    #[error("Failed to send eof to scp: {0}")]
    ScpSendEof(ssh2::Error),
    #[error("Failed to wait on eof on scp: {0}")]
    ScpWaitEof(std::io::Error),
    #[error("Timed out waiting for scp eof.")]
    ScpEndOfFileTimeout,
    #[error("Failed to close scp: {0}")]
    ScpClose(std::io::Error),
    #[error("Failed to wait on close on scp: {0}")]
    ScpWaitClose(std::io::Error),
    #[error("Failed to exec command: {0}")]
    Exec(ExecError),
    #[error("Decompress timed out.")]
    DecompressTimeout,
    #[error("Failed to decompress archive: {0}")]
    DecompressFailed(i32),
    #[error("Failed to terminate instances: {0}")]
    TerminateInstances(SdkError<aws_sdk_ec2::operation::terminate_instances::TerminateInstancesError>),
    #[error("Failed to delete key pair: {0}")]
    DeleteKeyPair(SdkError<aws_sdk_ec2::operation::delete_key_pair::DeleteKeyPairError>),
    // #[error("Failed to delete network interface: {0}")]
    // DeleteNetworkInterface(SdkError<aws_sdk_ec2::operation::delete_network_interface::DeleteNetworkInterfaceError>),
    // #[error("Failed to delete security group: {0}")]
    // DeleteSecurityGroup(SdkError<aws_sdk_ec2::operation::delete_security_group::DeleteSecurityGroupError>),
}

#[derive(Debug, thiserror::Error)]
enum ExecError {
    #[error("Failed to create channel: {0}")]
    Channel(std::io::Error),
    #[error("Timed out creating channel.")]
    ChannelTimeout,
    #[error("Failed exec: {0}")]
    Exec(std::io::Error),
    #[error("Timed out running exec.")]
    ExecTimeout,
    #[error("Failed to read stdout: {0}")]
    Stdout(std::io::Error),
    #[error("Failed to read stderr: {0}")]
    Stderr(std::io::Error),
    #[error("Failed to close channel: {0}")]
    Close(std::io::Error),
    #[error("Failed to get exit code: {0}")]
    Exit(ssh2::Error),
}

#[tokio::main]
async fn main() -> ExitCode {
    match main_exec() {
        Err(err) => {
            eprintln!("Error: {err:?}");
            ExitCode::FAILURE
        }
        Ok(None) => {
            eprintln!("Command timeout");
            ExitCode::FAILURE
        }
        Ok(Some(code)) => ExitCode::from(u8::try_from(code).unwrap()),
    }
}

#[tokio::main]
async fn main_exec() -> Result<Option<i32>, MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    tracing_subscriber::fmt().init();

    let (key_name, timeout, security_group_name, instance_type, ami, command, path, size) =
        parse_args();

    info!("Loading aws config");
    let config = aws_config::load_from_env().await;
    let client = ec2::Client::new(&config);

    info!("Creating SSH key pair");
    let builder = client.create_key_pair().key_name(key_name.clone());
    let create_key_pair_response = builder.send().await.map_err(CreateKeyPair)?;

    // Private key
    let key_material = create_key_pair_response
        .key_material
        .ok_or(CreateKeyPairMaterial)?;

    info!("Creating security groups");
    // The default settings prevent SSH working.
    let builder = client
        .create_security_group()
        .set_group_name(Some(security_group_name.clone()))
        .set_description(Some(String::from(SECURITY_GROUP_DESCRIPTION)));
    let create_security_group_response = builder.send().await.map_err(CreateSecurityGroup)?;
    let security_group_id = create_security_group_response
        .group_id
        .ok_or(CreateSecurityGroupId)?;

    // Set inbound rule (the default outbound rule is fine).
    info!("Setting ingress security group rule");
    let builder = client
        .authorize_security_group_ingress()
        .set_group_id(Some(security_group_id.clone()))
        .set_ip_protocol(Some(String::from("tcp")))
        .set_from_port(Some(22))
        .set_to_port(Some(22))
        .set_cidr_ip(Some(String::from("0.0.0.0/0")));
    builder
        .send()
        .await
        .map_err(AuthorizeSecurityGroupIngress)?;

    let code = run_instance(
        &client,
        &key_name,
        &security_group_name,
        &timeout,
        &path,
        &key_material,
        &command,
        &size,
        &instance_type,
        &ami,
    )
    .await?;
    info!("code: {code:?}");

    info!("Deleting key pair");
    let builder = client
        .delete_key_pair()
        .set_key_name(Some(key_name.clone()));
    builder.send().await.map_err(DeleteKeyPair)?;

    // TODO: Delete the created security group.
    // See the below commented out code.

    // info!("Deleting network interface");
    // let network_interface_id = todo!();
    // let builder = client.delete_network_interface().set_network_interface_id(input);
    // builder.send().await.map_err(DeleteNetworkInterface)?;

    // info!("Sleeping for {DELETE_SECURITY_GROUP_BUFFER:?}.");
    // sleep(DELETE_SECURITY_GROUP_BUFFER);

    // info!("Deleting security group");
    // let builder = client
    //     .delete_security_group()
    //     .set_group_id(Some(security_group_id.clone()));
    // builder.send().await.map_err(DeleteSecurityGroup)?;

    Ok(code)
}

fn parse_args() -> (
    String,
    Duration,
    String,
    InstanceType,
    String,
    String,
    Option<String>,
    VolumeSize,
) {
    info!("Parsing command line arguments");
    let args = Args::parse();

    let key_name = args
        .key_name
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let timeout = Duration::from_secs(args.timeout.unwrap_or(DEFAULT_COMMAND_TIMEOUT_SECS));
    let security_group_name = args
        .security_group_name
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let command = args
        .command
        .unwrap_or_else(|| String::from(DEFAULT_COMMAND));
    let path = args.path;
    let size = args.size.unwrap_or(DEFAULT_SIZE);

    (
        key_name,
        timeout,
        security_group_name,
        args.instance,
        args.ami,
        command,
        path,
        size,
    )
}

#[allow(clippy::too_many_arguments)]
async fn run_instance(
    client: &ec2::Client,
    key_name: &str,
    security_group_id: &str,
    timeout: &Duration,
    path: &Option<String>,
    private_key: &str,
    command: &str,
    size: &VolumeSize,
    instance: &InstanceType,
    ami: &str,
) -> Result<Option<i32>, MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    // Launches instance
    let (public_ip_address, instance_id) = launch_instance(
        client,
        instance,
        ami,
        key_name,
        security_group_id,
        timeout,
        size,
    )
    .await?;

    let ssh = create_ssh(&public_ip_address, timeout, private_key)?;
    let remote_path = format!("/tmp/{}", uuid::Uuid::new_v4());

    // Transfers source code
    if let Some(path) = path {
        transfer_source(path, &remote_path, &ssh, timeout).await?;
    }

    let code = exec(&ssh, command, timeout).map_err(Exec)?;

    info!("Sleeping for {RUN_BUFFER:?}.");
    sleep(RUN_BUFFER);

    info!("Terminate instances");
    let builder = client
        .terminate_instances()
        .set_instance_ids(Some(vec![instance_id]));
    builder.send().await.map_err(TerminateInstances)?;

    Ok(code)
}

fn create_ssh(
    public_ip_address: &str,
    timeout: &Duration,
    private_key: &str,
) -> Result<ssh2::Session, MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    // I have no idea why this is needed but for some reason we need to wait for ssh to work, I
    // don't know what is being waited on, this should poll.
    info!("Sleeping for {SSH_STARTUP_BUFFER:?}.");
    sleep(SSH_STARTUP_BUFFER);

    info!("Connecting SSH");
    let ipv4_address = std::net::Ipv4Addr::from_str(public_ip_address).map_err(PublicIpParse)?;
    let socket_address =
        std::net::SocketAddr::V4(std::net::SocketAddrV4::new(ipv4_address, EC2_SSH_PORT));
    info!("socket_address: {socket_address}");
    let tcp = std::net::TcpStream::connect(socket_address).map_err(TcpStreamConnect)?;
    tcp.set_nonblocking(true).unwrap();
    let mut ssh = ssh2::Session::new().map_err(SshSession)?;
    ssh.set_tcp_stream(tcp);
    ssh.set_blocking(false);

    // SSH handshake
    let start = Instant::now();
    info!("SSH handshake");
    loop {
        match ssh.handshake().map_err(std::io::Error::from) {
            Ok(()) => break,
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                if start.elapsed() > *timeout {
                    return Err(SshHandshakeTimeout);
                }
            }
            Err(err) => return Err(SshHandshake(err)),
        }
    }

    // SSH authorize
    let start = Instant::now();
    info!("SSH authorize");
    loop {
        match ssh
            .userauth_pubkey_memory("ubuntu", None, private_key, None)
            .map_err(std::io::Error::from)
        {
            Ok(()) => break,
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                if start.elapsed() > *timeout {
                    return Err(SshHandshakeTimeout);
                }
            }
            Err(err) => return Err(SshAuthSetup(err)),
        }
    }

    if !ssh.authenticated() {
        return Err(SshAuthFailed);
    }
    Ok(ssh)
}

/// Waits until the given instance is in the running state.
async fn wait_until_running(
    client: &ec2::Client,
    timeout: &Duration,
    instance_id: &str,
) -> Result<(), MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    let start = Instant::now();
    info!("Waiting for instance to enter the `running` state");
    loop {
        if start.elapsed() > *timeout {
            return Err(StartupTimeout);
        }

        sleep(INSTANCE_POLL_STATE_SLEEP);

        let builder = client
            .describe_instance_status()
            .set_instance_ids(Some(vec![instance_id.to_string()]))
            // By default this doesn't return descriptions for instances outside the `running`
            // state, in our case we want these description, so we set this parameter.
            .set_include_all_instances(Some(true));
        let describe_instance_status_response =
            builder.send().await.map_err(DescribeInstanceStatus)?;

        let Some(
            [ec2::types::InstanceStatus {
                instance_state:
                    Some(ec2::types::InstanceState {
                        name: Some(state), ..
                    }),
                ..
            }],
        ) = describe_instance_status_response
            .instance_statuses
            .as_deref()
        else {
            return Err(DescribeInstanceStatusState);
        };
        if *state == ec2::types::InstanceStateName::Running {
            return Ok(());
        }
    }
}

/// Using the default as recommend here
/// <https://docs.rs/aws-sdk-ec2/0.33.0/aws_sdk_ec2/types/builders/struct.BlockDeviceMappingBuilder.html#method.set_device_name>
/// and here <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/device_naming.html>.
const DEFAULT_BLOCK_DEVICE_NAME: &str = "/dev/sdh";

/// Launches an EC2 instance and returns the public ip address.
async fn launch_instance(
    client: &ec2::Client,
    instance_type: &InstanceType,
    ami: &str,
    key_name: &str,
    security_group_id: &str,
    timeout: &Duration,
    size: &VolumeSize,
) -> Result<(String, String), MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    info!("Launching instances");
    let builder = client
        .run_instances()
        .set_instance_type(Some(instance_type.clone()))
        .set_image_id(Some(String::from(ami)))
        .set_max_count(Some(1))
        .set_min_count(Some(1))
        .set_key_name(Some(String::from(key_name)))
        .set_security_group_ids(Some(vec![String::from(security_group_id)]))
        .set_block_device_mappings(Some(vec![aws_sdk_ec2::types::BlockDeviceMapping::builder(
        )
        .ebs(
            aws_sdk_ec2::types::EbsBlockDevice::builder()
                .set_volume_size(Some(i32::from(*size)))
                .build(),
        )
        .set_device_name(Some(String::from(DEFAULT_BLOCK_DEVICE_NAME)))
        .build()]));
    let run_instances_response = builder.send().await.map_err(RunInstances)?;

    let Some(
        [aws_sdk_ec2::types::Instance {
            instance_id: Some(instance_id),
            ..
        }],
    ) = &run_instances_response.instances.as_deref()
    else {
        return Err(RunInstancesInstanceId);
    };

    // The instance is not immediately assigned a public IP address so we need to wait.
    wait_until_running(client, timeout, instance_id.as_str()).await?;

    info!("Getting running instance description");
    let builder = client
        .describe_instances()
        .set_instance_ids(Some(vec![instance_id.clone()]));
    let describe_instances_response = builder.send().await.map_err(DescribeInstances)?;
    let public_ip_address = match describe_instances_response.reservations.as_deref() {
        Some(
            [ec2::types::Reservation {
                instances: Some(instance_descriptions),
                ..
            }],
        ) if let [ec2::types::Instance {
            public_ip_address: Some(public_ip_address),
            ..
        }] = instance_descriptions.as_slice() =>
        {
            public_ip_address
        }
        _ => return Err(DescribeInstancesPublicIpAddress),
    };

    Ok((public_ip_address.clone(), instance_id.clone()))
}

async fn get_archive_data(dir: &str) -> Result<&[u8], MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    static ARCHIVE: tokio::sync::OnceCell<Vec<u8>> = tokio::sync::OnceCell::const_new();
    let init = async || -> Result<Vec<u8>, MainError> {
        let mut tar_gz = Vec::new();
        let enc = GzEncoder::new(&mut tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);
        let ignore: [&str; 3] = ["/.git", "/.gitignore", "/target"];
        info!("Reading local directory: {dir:?}");
        let paths = std::fs::read_dir(dir).map_err(ReadDir)?;
        for path in paths {
            let entry = path.map_err(ReadEntry)?;
            let path_buf = entry.path();
            let name = path_buf.as_path().file_name().unwrap().to_str().unwrap();
            let path_string = path_buf.display().to_string();
            info!("Inspecting: {path_string:?}");
            if !ignore.contains(&path_string.as_str()) {
                let file_type = entry.file_type().map_err(ReadFileType)?;
                if file_type.is_dir() {
                    tar.append_dir_all(name, &path_string).map_err(AppendDir)?;
                } else if file_type.is_file() {
                    tar.append_path_with_name(path_string, name)
                        .map_err(AppendFile)?;
                }
            }
        }
        tar.into_inner().map_err(CompleteArchive)?;

        Ok(tar_gz)
    };
    ARCHIVE.get_or_try_init(init).await.map(Vec::as_slice)
}

/// Transfers source files to the instance
async fn transfer_source(
    local_path: &str,
    remote_path: &str,
    ssh: &ssh2::Session,
    timeout: &Duration,
) -> Result<(), MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    // Get source code data when stored in an archive.
    let data = get_archive_data(local_path).await.unwrap();

    info!("Copying source");

    // TODO What is the mode value of `0o644` doing here? I just copied it from the docs
    // https://docs.rs/ssh2/latest/ssh2/#upload-a-file.
    let start = Instant::now();
    info!("scp send");
    let mut channel = loop {
        if start.elapsed() > *timeout {
            return Err(SshHandshakeTimeout);
        }

        match ssh
            .scp_send(Path::new(remote_path), 0o644, data.len() as u64, None)
            .map_err(std::io::Error::from)
        {
            Ok(c) => break c,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(ScpSend(err)),
        }
    };

    // Write archive to remote.
    let mut n = 0;
    loop {
        if start.elapsed() > *timeout {
            return Err(SshHandshakeTimeout);
        }
        n += match channel.write(&data[n..]) {
            Ok(0) if n == data.len() => break,
            Ok(c) => c,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(ScpWrite(err)),
        };
    }

    // Wait send ending for write of archive to remote.
    channel.send_eof().map_err(ScpSendEof)?;

    // Wait for end of file
    let start = Instant::now();
    info!("Wait for scp end of file");
    loop {
        if start.elapsed() > *timeout {
            return Err(ScpEndOfFileTimeout);
        }

        match channel.wait_eof().map_err(std::io::Error::from) {
            Ok(()) => break,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(ScpWaitEof(err)),
        }
    }

    info!("Closing scp");
    loop {
        if start.elapsed() > *timeout {
            return Err(ScpEndOfFileTimeout);
        }

        match channel.close().map_err(std::io::Error::from) {
            Ok(()) => break,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(ScpClose(err)),
        }
    }

    info!("Waiting close scp");
    loop {
        if start.elapsed() > *timeout {
            return Err(ScpEndOfFileTimeout);
        }

        match channel.wait_close().map_err(std::io::Error::from) {
            Ok(()) => break,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(ScpWaitClose(err)),
        }
    }

    info!("Decompressing source");

    let Some(code) = exec(ssh, &format!("tar -xf {remote_path}"), timeout).map_err(Exec)? else {
        return Err(DecompressTimeout);
    };
    if code != 0 {
        return Err(DecompressFailed(code));
    }
    Ok(())
}

fn exec(
    session: &ssh2::Session,
    command: &str,
    timeout: &Duration,
) -> Result<Option<i32>, ExecError> {
    #[allow(clippy::enum_glob_use)]
    use ExecError::*;

    let start = std::time::Instant::now();

    // Opening channel
    info!("Opening channel");
    let mut channel = loop {
        if start.elapsed() > *timeout {
            return Err(ChannelTimeout);
        }

        match session.channel_session().map_err(std::io::Error::from) {
            Ok(c) => break c,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(Channel(err)),
        }
    };

    // Get stderr stream
    let mut stderr_stream = channel.stderr();

    // Running exec
    info!("Running exec: {command:?}");
    loop {
        if start.elapsed() > *timeout {
            return Err(ExecTimeout);
        }

        match channel.exec(command).map_err(std::io::Error::from) {
            Ok(()) => break,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(Exec(err)),
        }
    }

    // Stdout
    // ---------------------------------------------------------------------------------------------
    let timeout_stdout = *timeout;
    let stdout_handle = std::thread::spawn(move || {
        let mut stdout = std::io::stdout();
        loop {
            let mut buffer = [u8::default(); 1024];
            let n = match channel.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) if err.kind() == WouldBlock => continue,
                Err(err) => return Err(Stdout(err)),
            };

            stdout.write_all(&buffer[..n]).unwrap();
        }

        Ok(channel)
    });

    // Stderr
    // ---------------------------------------------------------------------------------------------
    let timeout_stderr = *timeout;
    let stderr_handle = std::thread::spawn(move || {
        let mut stderr = std::io::stderr();
        loop {
            if start.elapsed() > timeout_stderr {
                break;
            }

            let mut buffer = [u8::default(); 1024];
            let n = match stderr_stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) if err.kind() == WouldBlock => continue,
                Err(err) => return Err(Stderr(err)),
            };

            stderr.write_all(&buffer[..n]).unwrap();
        }
        Ok(())
    });

    // Wait
    // ---------------------------------------------------------------------------------------------
    let (stdout_result, stderr_result) = (stdout_handle.join(), stderr_handle.join());

    let mut channel = stdout_result.unwrap()?;
    info!("Joined stdout");
    stderr_result.unwrap()?;
    info!("Joined stderr");

    // Exit
    // ---------------------------------------------------------------------------------------------
    info!("Waiting on close");
    loop {
        if start.elapsed() > timeout_stdout {
            return Ok(None);
        }

        match channel.wait_close().map_err(std::io::Error::from) {
            Ok(()) => return Ok(Some(channel.exit_status().map_err(Exit)?)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => return Err(Close(err)),
        }
    }
}
