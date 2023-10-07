#![feature(if_let_guard)]
#![warn(clippy::pedantic)]
#![feature(let_chains)]
#![feature(async_closure)]

use aws_sdk_ec2 as ec2;
use clap::Parser;
use ec2::types::InstanceType;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::OpenOptions;
use std::io::ErrorKind::WouldBlock;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use tracing::info;

/// <https://github.com/libssh2/libssh2/blob/master/include/libssh2.h>
const LIBSSH2_ERROR_EAGAIN: i32 = -37;
const SSH2_WOULD_BLOCK: ssh2::ErrorCode = ssh2::ErrorCode::Session(LIBSSH2_ERROR_EAGAIN);

/// t2.micro
/// Ubuntu 22.04 LTS
const DEFAULT_INSTANCES: [(InstanceType, &str); 2] = [
    (InstanceType::T2Medium, "ami-0eb260c4d5475b901"),
    (InstanceType::T4gMedium, "ami-0e3f80b3d2a794117"),
];

/// The default key pair name.
const DEFAULT_KEY_NAME: &str = "test-aws-key-pair-2";

/// The default port used by ec2 for ssh.
const EC2_SSH_PORT: u16 = 22;

// TODO This should only be default for optional command line argument.
const INSTANCE_POLL_STATE_SLEEP: Duration = Duration::from_secs(1);

// TODO This should only be default for optional command line argument.
const DEFAULT_SECURITY_GROUP_NAME: &str = "test-aws-security-group-2";

// TODO This should only be default for optional command line argument.
const SECURITY_GROUP_DESCRIPTION: &str = "test-aws-security-group-description";

/// The default archive name.
const DEFAULT_ARCHIVE_NAME: &str = "archive.tar.gz";

/// Default command to run on the host.
const DEFAULT_COMMAND: &str = "cat /proc/cpuinfo && ls";

// TODO Remove this.
const RUN_BUFFER: Duration = Duration::from_secs(30);

// TODO Replace this with polling.
const SSH_STARTUP_BUFFER: Duration = Duration::from_secs(20);

// TODO Replace this with polling so it doesn't need to wait longer than neccessary.
/// We need to wait a long time for the instance to be terminated and for the security group to lose
/// its dependency so it can be deleted.
const DELETE_SECURITY_GROUP_BUFFER: Duration = Duration::from_secs(120);

const DEFAULT_COMMAND_TIMEOUT_SECS: u64 = 300;

const DEFAULT_PATH: &str = "./";

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    path: Option<String>,
    /// Name of the SSH key pair used.
    #[arg(long)]
    key_name: Option<String>,
    /// The name of the archive used to compress and transfer the source code in.
    #[arg(long)]
    archive: Option<String>,
    /// The name to use for the security group for instances.
    #[arg(long)]
    security_group_name: Option<String>,
    /// Timeout in seconds.
    #[arg(long)]
    timeout: Option<u64>,
    /// The command to run on the instance.
    #[arg(long)]
    command: Option<String>,
    /// The EC2 instance types and AMI to use to launch instances.
    #[arg(long, value_delimiter = ',')]
    instances: Vec<InstanceType>,
    /// The EC2 instance types and AMI to use to launch instances.
    #[arg(long, value_delimiter = ',')]
    amis: Vec<String>,
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
    SshHandshake(ssh2::Error),
    #[error("Timed out attempting SSH handshake.")]
    SshHandshakeTimeout,
    #[error("Failed to setup SSH auth: {0}")]
    SshAuthSetup(ssh2::Error),
    #[error("Failed SSH auth.")]
    SshAuthFailed,
    #[error("Failed to create source archive: {0}")]
    CreateArchive(std::io::Error),
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
    ScpSend(ssh2::Error),
    #[error("Failed to write to scp: {0}")]
    ScpWrite(std::io::Error),
    #[error("Failed to send eof to scp: {0}")]
    ScpSendEof(ssh2::Error),
    #[error("Failed to wait on eof on scp: {0}")]
    ScpWaitEof(ssh2::Error),
    #[error("Timed out waiting for scp eof.")]
    ScpEndOfFileTimeout,
    #[error("Failed to close scp: {0}")]
    ScpClose(ssh2::Error),
    #[error("Failed to wait on close on scp: {0}")]
    ScpWaitClose(ssh2::Error),
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
    #[error("Failed to delete security group: {0}")]
    DeleteSecurityGroup(SdkError<aws_sdk_ec2::operation::delete_security_group::DeleteSecurityGroupError>),
}

#[derive(Debug, thiserror::Error)]
enum ExecError {
    #[error("Failed to create channel: {0}")]
    Channel(ssh2::Error),
    #[error("Timed out creating channel.")]
    ChannelTimeout,
    #[error("Failed exec: {0}")]
    Exec(ssh2::Error),
    #[error("Timed out running exec.")]
    ExecTimeout,
    #[error("Failed to read stdout: {0}")]
    Stdout(std::io::Error),
    #[error("Failed to read stderr: {0}")]
    Stderr(std::io::Error),
    #[error("Failed to close channel: {0}")]
    Close(ssh2::Error),
    #[error("Failed to get exit code: {0}")]
    Exit(ssh2::Error),
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::fmt().with_thread_ids(true).init();

    let (key_name, timeout, security_group_name, instances, command, path) = parse_args();

    let result = create_resources(
        key_name,
        instances,
        security_group_name,
        timeout,
        command,
        path,
    )
    .await?;

    println!("results: {result:?}");

    Ok(())
}

fn parse_args() -> (
    String,
    Duration,
    String,
    Vec<(InstanceType, String)>,
    String,
    String,
) {
    info!("Parsing command line arguments");
    let args = Args::parse();

    let key_name = args
        .key_name
        .unwrap_or_else(|| String::from(DEFAULT_KEY_NAME));
    let timeout = Duration::from_secs(args.timeout.unwrap_or(DEFAULT_COMMAND_TIMEOUT_SECS));
    let security_group_name = args
        .security_group_name
        .unwrap_or_else(|| String::from(DEFAULT_SECURITY_GROUP_NAME));

    let instances = {
        assert_eq!(args.instances.len(), args.amis.len());
        let mut instances = args
            .instances
            .into_iter()
            .zip(args.amis)
            .collect::<Vec<_>>();
        if instances.is_empty() {
            instances = DEFAULT_INSTANCES
                .iter()
                .cloned()
                .map(|(i, a)| (i, String::from(a)))
                .collect();
        }
        instances
    };

    let command = args
        .command
        .unwrap_or_else(|| String::from(DEFAULT_COMMAND));
    let path = args.path.unwrap_or_else(|| String::from(DEFAULT_PATH));

    (
        key_name,
        timeout,
        security_group_name,
        instances,
        command,
        path,
    )
}

#[allow(clippy::too_many_lines)]
async fn create_resources(
    key_name: String,
    instances: Vec<(ec2::types::InstanceType, String)>,
    security_group_name: String,
    timeout: Duration,
    command: String,
    path: String,
) -> Result<Vec<Option<i32>>, MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

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
        .set_group_name(Some(security_group_name))
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

    // TODO To check the rule is added to the security group, we need to check the rule is present
    // in the response list of rules added. Do this check.

    // Create instances
    let mut handles = Vec::new();

    // Do to issues with tokio and rustlang, it is not smart enough to figure out passing
    // referneces is fine so we need to clone this data.
    let data = Arc::new((
        client,
        key_name,
        security_group_id,
        timeout,
        path,
        key_material,
        command,
    ));

    for (instance_type, ami) in instances {
        let data_clone = data.clone();
        let handle =
            tokio::task::spawn(async { run_instance(data_clone, instance_type, ami).await });
        handles.push(handle);
    }

    let mut codes = Vec::new();
    for handle in handles {
        let code = handle.await.unwrap()?;
        codes.push(code);
    }

    let (client, key_name, security_group_id, _timeout, _path, _key_material, _command) = &*data;

    info!("Deleting key pair");
    let builder = client
        .delete_key_pair()
        .set_key_name(Some(key_name.clone()));
    builder.send().await.map_err(DeleteKeyPair)?;

    info!("Sleeping for {DELETE_SECURITY_GROUP_BUFFER:?}.");
    sleep(DELETE_SECURITY_GROUP_BUFFER);

    info!("Deleting security group");
    let builder = client
        .delete_security_group()
        .set_group_id(Some(security_group_id.clone()));
    builder.send().await.map_err(DeleteSecurityGroup)?;

    Ok(codes)
}

async fn run_instance(
    data: Arc<(
        ec2::Client,
        String,
        String,
        Duration,
        String,
        String,
        String,
    )>,
    instance_type: InstanceType,
    ami: String,
) -> Result<Option<i32>, MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    let (client, key_name, security_group_id, timeout, path, private_key, command) = &*data;

    // Launches instance
    let (public_ip_address, instance_id) = launch_instance(
        client,
        instance_type,
        ami,
        key_name,
        security_group_id,
        timeout,
    )
    .await?;

    let ssh = create_ssh(&public_ip_address, timeout, private_key)?;

    // Transfers source code
    transfer_source(path, &ssh, timeout).await?;

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
        match ssh.handshake() {
            Ok(()) => break,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => {
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
        match ssh.userauth_pubkey_memory("ubuntu", None, private_key, None) {
            Ok(()) => break,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => {
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

/// Launches an EC2 instance and returns the public ip address.
async fn launch_instance(
    client: &ec2::Client,
    instance_type: InstanceType,
    ami: String,
    key_name: &str,
    security_group_id: &str,
    timeout: &Duration,
) -> Result<(String, String), MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    info!("Launching instances");
    let builder = client
        .run_instances()
        .set_instance_type(Some(instance_type))
        .set_image_id(Some(ami))
        .set_max_count(Some(1))
        .set_min_count(Some(1))
        .set_key_name(Some(String::from(key_name)))
        .set_security_group_ids(Some(vec![String::from(security_group_id)]));
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
            }]) if let [ec2::types::Instance {
                public_ip_address: Some(public_ip_address),
                ..
            }] = instance_descriptions.as_slice() => public_ip_address,
            _ => return Err(DescribeInstancesPublicIpAddress),
    };

    Ok((public_ip_address.clone(), instance_id.clone()))
}

async fn get_archive_data(dir: &str) -> Result<&[u8], MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    static ARCHIVE: tokio::sync::OnceCell<Vec<u8>> = tokio::sync::OnceCell::const_new();
    let init = async || -> Result<Vec<u8>, MainError> {
        info!("Compressing source");
        let tar_gz = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(DEFAULT_ARCHIVE_NAME)
            .map_err(CreateArchive)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);
        let ignore: [String; 4] = [
            String::from("./.git"),
            String::from("./.gitignore"),
            String::from("./target"),
            format!("./{DEFAULT_ARCHIVE_NAME}"),
        ];
        let paths = std::fs::read_dir(dir).map_err(ReadDir)?;
        for path in paths {
            let entry = path.map_err(ReadEntry)?;
            let path_buf = entry.path();
            let path_string = path_buf.display().to_string();
            if !ignore.contains(&path_string) {
                let file_type = entry.file_type().map_err(ReadFileType)?;
                if file_type.is_dir() {
                    tar.append_dir_all(&path_string, &path_string)
                        .map_err(AppendDir)?;
                } else if file_type.is_file() {
                    tar.append_path(path_string).map_err(AppendFile)?;
                }
            }
        }
        tar.into_inner().map_err(CompleteArchive)?;

        let mut file = OpenOptions::new()
            .read(true)
            .open(DEFAULT_ARCHIVE_NAME)
            .unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        Ok(buffer)
    };
    ARCHIVE.get_or_try_init(init).await.map(Vec::as_slice)
}

/// Transfers source files to the instance
async fn transfer_source(
    path: &str,
    ssh: &ssh2::Session,
    timeout: &Duration,
) -> Result<(), MainError> {
    #[allow(clippy::enum_glob_use)]
    use MainError::*;

    // Get source code data when stored in an archive.
    let data = get_archive_data(path).await.unwrap();

    info!("Copying source");

    // TODO What is the mode value of `0o644` doing here? I just copied it from the docs
    // https://docs.rs/ssh2/latest/ssh2/#upload-a-file.
    let start = Instant::now();
    info!("scp send");
    let mut channel = loop {
        if start.elapsed() > *timeout {
            return Err(SshHandshakeTimeout);
        }

        match ssh.scp_send(
            Path::new(DEFAULT_ARCHIVE_NAME),
            0o644,
            data.len() as u64,
            None,
        ) {
            Ok(c) => break c,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => continue,
            Err(err) => return Err(ScpSend(err)),
        }
    };

    // Write archive to remote.
    channel.write_all(data).map_err(ScpWrite)?;

    // Wait send ending for write of archive to remote.
    channel.send_eof().map_err(ScpSendEof)?;

    // Wait for end of file
    let start = Instant::now();
    info!("Wait for scp end of file");
    loop {
        if start.elapsed() > *timeout {
            return Err(ScpEndOfFileTimeout);
        }

        match channel.wait_eof() {
            Ok(()) => break,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => continue,
            Err(err) => return Err(ScpWaitEof(err)),
        }
    }

    info!("Closing scp");
    loop {
        if start.elapsed() > *timeout {
            return Err(ScpEndOfFileTimeout);
        }

        match channel.close() {
            Ok(()) => break,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => continue,
            Err(err) => return Err(ScpClose(err)),
        }
    }

    info!("Waiting close scp");
    loop {
        if start.elapsed() > *timeout {
            return Err(ScpEndOfFileTimeout);
        }

        match channel.wait_close() {
            Ok(()) => break,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => continue,
            Err(err) => return Err(ScpWaitClose(err)),
        }
    }

    info!("Decompressing source");
    let Some(code) =
        exec(ssh, &format!("tar -xf {DEFAULT_ARCHIVE_NAME}"), timeout).map_err(Exec)?
    else {
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

        match session.channel_session() {
            Ok(c) => break c,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => continue,
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

        match channel.exec(command) {
            Ok(()) => break,
            Err(err) if err.code() == SSH2_WOULD_BLOCK => continue,
            Err(err) => return Err(Exec(err)),
        }
    }

    // Stdout
    // ---------------------------------------------------------------------------------------------
    let timeout_stdout = *timeout;
    let stdout_handle = std::thread::spawn(move || {
        let mut stdout = Vec::new();
        loop {
            if start.elapsed() > timeout_stdout {
                break;
            }

            let mut buffer = [u8::default(); 1024];
            let n = match channel.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) if err.kind() == WouldBlock => continue,
                Err(err) => return Err(Stdout(err)),
            };

            stdout.write_all(&buffer[..n]).unwrap();
        }

        Ok((channel, stdout))
    });

    // Stderr
    // ---------------------------------------------------------------------------------------------
    let timeout_stderr = *timeout;
    let stderr_handle = std::thread::spawn(move || {
        let mut stderr = Vec::new();
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
        Ok(stderr)
    });

    // Wait
    // ---------------------------------------------------------------------------------------------
    let (mut channel, stdout) = stdout_handle.join().unwrap()?;
    info!("stdout: {:?}", std::str::from_utf8(&stdout).unwrap());
    let stderr = stderr_handle.join().unwrap()?;
    info!("stderr: {:?}", std::str::from_utf8(&stderr).unwrap());

    // Exit
    // ---------------------------------------------------------------------------------------------
    info!("Waiting on close");
    loop {
        if start.elapsed() > timeout_stdout {
            return Ok(None);
        }

        match channel.wait_close() {
            Ok(()) => return Ok(Some(channel.exit_status().map_err(Exit)?)),
            Err(err) if err.code() == SSH2_WOULD_BLOCK => continue,
            Err(err) => return Err(Close(err)),
        }
    }
}
