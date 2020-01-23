use std::io::prelude::*;
use std::io::{Write, Seek, copy, Error, ErrorKind};
use std::{error};
use std::path::Path;
use std::fs::{File, create_dir_all};
use std::net::{TcpStream};

use tempfile::Builder;
use log::{debug, info};

mod zip_helper;

pub fn request_compilation(source_folder: &str, output_folder: &str, host: &str) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    // Check that the directory is a valid SPTCloud project
    debug!("Verifying that {} is a valid project", source_folder);
    let source_path = Path::new(source_folder);
    let sptcloud_manifest_path = source_path.join(".sptcloud.json");
    if File::open(sptcloud_manifest_path).is_err() {
        let error = Error::new(ErrorKind::InvalidInput, format!("'{}' is not a valid SPTCloud project", source_folder));
        return Err(Box::new(error));
    }

    // Build the archive file path
    let build_dir = Builder::new().prefix("sptcloud-request-").tempdir_in(".")?;
    let archive_file_path = build_dir.path().join("archive.zip");
    let archive_file = archive_file_path.to_str().ok_or("Archive file path invalid")?;

    // Build the output path
    let output_path = Path::new(output_folder);
    let output_file_path = output_path.join("main.pdf");
    let output_file = output_file_path.to_str().ok_or("Output file path invalid")?;

    // Compress the project
    if let Some(project_name) = source_path.file_name() {
        info!("Compressing project {:?}", project_name);
    } else {
        info!("Compressing project");
    }

    zip_helper::compress_folder(source_folder, archive_file, zip::CompressionMethod::Deflated)?;

    // Create the output directory and send the request
    create_dir_all(output_path)?;
    send_request(archive_file, output_file, host)?;

    Ok(())
}

fn send_request(source_archive: &str, destination_file: &str, host: &str) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let mut file = File::open(source_archive)?;
    let file_size = file.stream_len()?;

    info!("Connecting to {}", host);
    let mut stream = TcpStream::connect(host)?;
    
    // Write the file size and content
    stream.write_all(&mut file_size.to_ne_bytes())?;
    debug!("Uploading {} bytes", file_size);
    copy(&mut file, &mut stream)?;
    stream.flush()?;

    // Read the response size
    info!("Waiting for build to complete ...");
    let mut response_size_buffer = [0; 8];
    stream.read_exact(&mut response_size_buffer)?;
    let response_size = u64::from_ne_bytes(response_size_buffer);

    // Read the response
    debug!("Downloading {} bytes", response_size);
    let mut response_handle = stream.take(response_size);
    let mut output_file = File::create(destination_file)?;
    copy(&mut response_handle, &mut output_file)?;

    Ok(())
}