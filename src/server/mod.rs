use std::{error};
use std::io::prelude::*;
use std::path::Path;
use std::io::{Write, Seek, copy};
use std::fs::{File};
use std::process::{Command};
use std::time::Duration;

use std::net::{TcpListener};
use std::thread;

use tempfile::Builder;
use uuid::Uuid;

use log::{debug, info, warn};

pub fn host(bind_address: &str) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let listener = TcpListener::bind(bind_address)?;

    warn!("SPTCloud listening on {}", bind_address);
    
    for stream in listener.incoming() {
        thread::spawn(|| {
            let mut stream = stream.unwrap();
            let stream_id = Uuid::new_v4();

            stream.set_read_timeout(Some(Duration::from_secs(300))).unwrap();

            if let Ok(peer_addr) = stream.peer_addr() {
                info!("Incoming connection from {} ({})", peer_addr, stream_id);
            } else {
                info!("Incoming connection ({})", stream_id);
            }
            
            // Read the incoming project archive size
            let mut archive_size_buffer = [0; 8];
            stream.read_exact(&mut archive_size_buffer).unwrap();
            let archive_size = u64::from_ne_bytes(archive_size_buffer);

            if archive_size > 100_000_000 {
                return;
            }

            // Read the archive
            debug!("Receiving project file ({} bytes)", archive_size);
            let dir = Builder::new().prefix("sptcloud-build-").tempdir_in(".").unwrap();
            let archive_file_path = dir.path().join("archive.zip");
            let mut archive_file = File::create(archive_file_path).unwrap();
            let mut handler = (&stream).take(archive_size);
            copy(&mut handler, &mut archive_file).unwrap();

            // Build the project
            execute_compilation_container(dir.path(), &stream_id);

            // Send the result size and content
            let output_file_path = dir.path().join("main.pdf");
            let mut output_file = File::open(output_file_path).unwrap();
            let output_size = output_file.stream_len().unwrap();
            debug!("Sending artifact ({} bytes)", output_size);
            
            // Write the file size and content
            stream.write_all(&mut output_size.to_ne_bytes()).unwrap();
            copy(&mut output_file, &mut stream).unwrap();

            dir.close().unwrap();
        });
    }

    Ok(())
}

fn execute_compilation_container(mount_directory: &std::path::Path, id: &Uuid) {
    let mount = format!("{}:/mount", mount_directory.to_str().unwrap());
    let log_file = File::create(format!("{}.log", id.to_simple().to_string())).unwrap();

    let status = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-it")
        .arg("-v")
        .arg(mount)
        .arg("docker.pkg.github.com/texnak/sptcloud/sptcloud-pandoc-build:latest")
        .stdout(log_file)
        .status()
        .expect("failed to execute process");

    assert!(status.success());
}