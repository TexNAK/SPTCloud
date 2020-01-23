use std::io::prelude::*;
use std::io::{Write, Seek, copy};
use std::{error};
use std::path::Path;
use std::fs::{File, create_dir_all};
use std::net::{TcpStream};

use tempfile::Builder;

mod zip_helper;

pub fn request_compilation() -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let src_dir = "../Science-Paper-Template";
    let build_dir = Builder::new().prefix("sptcloud-request-").tempdir_in(".")?;
    let archive_file_path = build_dir.path().join("archive.zip");
    let archive_file = archive_file_path.to_str().unwrap();

    let output_path = Path::new("./out");

    zip_helper::compress_folder(src_dir, archive_file, zip::CompressionMethod::Deflated)?;
    create_dir_all(output_path)?;
    send_one_command(archive_file, output_path.join("main.pdf").to_str().unwrap())?;

    Ok(())
}

const HOST: &'static str = "127.0.0.1:2399";

fn send_one_command(source_archive: &str, destination_file: &str) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let mut file = File::open(source_archive)?;
    let file_size = file.stream_len()?;

    let mut stream = TcpStream::connect(HOST)?;
    
    // Write the file size and content
    stream.write_all(&mut file_size.to_ne_bytes())?;
    copy(&mut file, &mut stream)?;

    // Read the response size
    let mut response_size_buffer = [0; 8];
    stream.read_exact(&mut response_size_buffer)?;
    let response_size = u64::from_ne_bytes(response_size_buffer);

    // Read the response
    let mut response_handle = stream.take(response_size);
    let mut output_file = File::create(destination_file)?;
    copy(&mut response_handle, &mut output_file)?;

    Ok(())
}