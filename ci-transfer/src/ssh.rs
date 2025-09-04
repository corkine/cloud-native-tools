use ssh2::Session;
use std::borrow::Cow;
use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, Instant};
use base64::{engine::general_purpose, Engine as _};

use crate::error::TransferError;
use crate::Args;

fn transfer_file(
    session: &Session,
    local_path: &Path,
    remote_path: &str,
) -> Result<(), TransferError> {
    let mut local_file = File::open(local_path)?;
    let file_size = local_file.metadata()?.len();
    let mut remote_file = session.scp_send(Path::new(remote_path), 0o644, file_size, None)?;

    let mut buffer = vec![0; 1024 * 1024]; // 1MB buffer
    let mut total_transferred = 0;
    let start_time = Instant::now();
    let mut last_update = Instant::now();

    loop {
        let bytes_read = local_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        remote_file.write_all(&buffer[..bytes_read])?;
        total_transferred += bytes_read as u64;

        // Update progress every second
        if last_update.elapsed() >= Duration::from_secs(1) {
            print_progress(total_transferred, file_size, start_time.elapsed());
            last_update = Instant::now();
        }
    }

    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    print_progress(total_transferred, file_size, start_time.elapsed());
    println!("\nTransferred: {:?} -> {}", local_path, remote_path);
    Ok(())
}

fn print_progress(transferred: u64, total: u64, elapsed: Duration) {
    let percentage = (transferred as f64 / total as f64) * 100.0;
    let speed = transferred as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0; // MB/s
    print!(
        "\rProgress: {:.2}% ({}/{} bytes) - {:.2} MB/s",
        percentage, transferred, total, speed
    );
    std::io::stdout().flush().unwrap();
}

fn transfer_directory(
    session: &Session,
    local_dir: &Path,
    remote_dir: &str,
) -> Result<(), TransferError> {
    let sftp = session.sftp()?;
    sftp.mkdir(Path::new(remote_dir), 0o755)?;

    for entry in read_dir(local_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let remote_path = format!("{}/{}", remote_dir, file_name);

        if path.is_dir() {
            transfer_directory(session, &path, &remote_path)?;
        } else {
            transfer_file(session, &path, &remote_path)?;
        }
    }

    Ok(())
}

pub fn transfer(session: &Session, sources: &[String], remote_path: &str) -> Result<(), TransferError> {
    // Handle empty sources case (side-effect only)
    if sources.is_empty() {
        println!("No source files specified for SSH transfer - side-effect only operation");
        return Ok(());
    }

    for source in sources {
        println!("Transferring: {} -> {}", source, remote_path);
        let source_path = Path::new(source);
        
        if !source_path.exists() {
            return Err(TransferError::Other(format!(
                "Source path {} does not exist",
                source
            )));
        }
        
        if source_path.is_dir() {
            let dir_name = source_path.file_name().unwrap().to_str().unwrap();
            let target_dir = if remote_path.ends_with('/') {
                format!("{}{}", remote_path, dir_name)
            } else {
                // For multiple sources, create subdirectories
                if sources.len() > 1 {
                    format!("{}/{}", remote_path.trim_end_matches('/'), dir_name)
                } else {
                    remote_path.to_string()
                }
            };
            transfer_directory(session, source_path, &target_dir)?;
        } else {
            let remote_file_path = if remote_path.ends_with('/') {
                format!(
                    "{}{}",
                    remote_path,
                    source_path.file_name().unwrap().to_str().unwrap()
                )
            } else {
                // For multiple files, we need to create unique paths
                if sources.len() > 1 {
                    let file_name = source_path.file_name().unwrap().to_str().unwrap();
                    format!("{}/{}", remote_path.trim_end_matches('/'), file_name)
                } else {
                    remote_path.to_string()
                }
            };
            transfer_file(session, source_path, &remote_file_path)?;
        }
    }
    
    Ok(())
}

pub fn execute_ssh_commands(session: &Session, commands: &[String]) -> Result<(), TransferError> {
    for command in commands {
        if command.is_empty() {
            continue;
        }
        if let Ok(decoded) = general_purpose::STANDARD.decode(&command) {
            if let Ok(decoded_str) = std::str::from_utf8(&decoded) {
                execute_ssh_commands(session, &[decoded_str.to_string()])?;
                continue;
            }
        }
        let mut channel = session.channel_session()?;
        let escaped_command = escape_command(command);
        let wrapped_command = format!("bash -c {}", escaped_command);
        channel.exec(&wrapped_command)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        println!("Command: {}", command);
        println!("Output: {}", output);
        channel.wait_close()?;
        println!("Exit status: {}", channel.exit_status()?);
        println!("---");
    }
    Ok(())
}

fn escape_command(cmd: &str) -> Cow<str> {
    if cmd.contains('"') || cmd.contains('\\') {
        let escaped = cmd.replace('"', "\\\"").replace('\\', "\\\\");
        Cow::Owned(format!("\"{}\"", escaped))
    } else {
        Cow::Borrowed(cmd)
    }
}

pub struct SshConfig {
    username: String,
    password: String,
    ip: String,
    remote_path: String,
}

pub fn parse_destination_ssh(destination: &str) -> Result<SshConfig, TransferError> {
    if destination.is_empty() {
        return Err(TransferError::Other("Destination cannot be empty".into()));
    }
    match general_purpose::STANDARD.decode(&destination) {
        Ok(decoded) => match std::str::from_utf8(&decoded) {
            Ok(s) => return parse_destination_ssh(s),
            _ => (),
        },
        _ => (),
    }
    let parts: Vec<&str> = destination.split('@').collect();
    if parts.len() != 2 {
        return Err(TransferError::Other("Invalid destination format".into()));
    }

    let credentials: Vec<&str> = parts[0].split(':').collect();
    if credentials.len() != 2 {
        return Err(TransferError::Other("Invalid credentials format".into()));
    }

    let server_info: Vec<&str> = parts[1].split(':').collect();
    if server_info.len() != 2 {
        return Err(TransferError::Other("Invalid server info format".into()));
    }

    Ok(SshConfig {
        username: credentials[0].to_string(),
        password: credentials[1].to_string(),
        ip: server_info[0].to_string(),
        remote_path: server_info[1].to_string(),
    })
}

pub fn handle_ssh(args: &Args, ssh_config: SshConfig) -> Result<(), TransferError> {
    let tcp = TcpStream::connect(format!("{}:{}", ssh_config.ip, args.port))?;
    println!("Connected to {}:{}", ssh_config.ip, args.port);
    let mut session = Session::new()?;
    session.set_timeout(0);
    session.set_tcp_stream(tcp);
    session.handshake()?;
    session.userauth_password(&ssh_config.username, &ssh_config.password)?;

    // Execute precommands if they exist
    if !args.precommands.is_empty() {
        println!("Executing pre-transfer commands:");
        execute_ssh_commands(&session, &args.precommands)?;
        println!("Pre-transfer commands completed.");
    }

    transfer(&session, &args.source, &ssh_config.remote_path)?;
    println!("\nFile(s) transferred successfully");

    if !args.commands.is_empty() {
        println!("Executing post-transfer commands:");
        execute_ssh_commands(&session, &args.commands)?;
        println!("Post-transfer commands completed.");
    }

    Ok(())
}