#![feature(seek_convenience)]

use std::{env, error};
use std::io::prelude::*;
use std::io::{Write, Seek, copy};
use std::iter::Iterator;
use zip::write::FileOptions;
use zip::result::ZipError;

use walkdir::{WalkDir, DirEntry};
use std::path::Path;
use std::fs::File;

use std::net::{TcpStream, TcpListener};
use std::thread;

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let args: Vec<_> = env::args().collect();

    if args.len() == 2 && args[1] == "server" {
        match host_server() {
            Ok(_) => println!("server exited."),
            Err(e) => println!("Error: {:?}", e),
        }
    } else {
        match compile_project() {
            Ok(_) => println!("done."),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    return 0;
}

fn host_server() -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:2399")?;
    
    println!("listening started, ready to accept");
    
    for stream in listener.incoming() {
        thread::spawn(|| {
            let mut stream = stream.unwrap();
            
            // Read the incoming project archive size
            let mut archive_size_buffer = [0; 8];
            stream.read_exact(&mut archive_size_buffer).unwrap();
            let archive_size = u64::from_ne_bytes(archive_size_buffer);

            if archive_size > 100_000_000 {
                return;
            }

            // Read the archive
            println!("Receiving project file ({} bytes)", archive_size);
            let mut archive_file = File::create("/tmp/received_archive.zip").unwrap();
            let mut handler = (&stream).take(archive_size);
            copy(&mut handler, &mut archive_file).unwrap();

            // TODO: Build the project

            // Send the result size and content
            let mut output_file = File::open("../Science-Paper-Template/out/main.pdf").unwrap();
            let output_size = output_file.stream_len().unwrap();
            println!("Sending artifact ({} bytes)", output_size);
            
            // Write the file size and content
            stream.write_all(&mut output_size.to_ne_bytes()).unwrap();
            copy(&mut output_file, &mut stream).unwrap();
        });
    }

    Ok(())
}

fn compile_project() -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let src_dir = "../Science-Paper-Template";
    let dst_file = "/tmp/output.zip";

    compress_folder(src_dir, dst_file, zip::CompressionMethod::Deflated)?;
    send_one_command(dst_file, "/tmp/out.pdf")?;

    Ok(())
}

fn zip_dir<T>(it: &mut dyn Iterator<Item=DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod) -> zip::result::ZipResult<()> where T: Write+Seek {
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let blacklist = [".git", ".github", ".vscode", "out", ".DS_Store"];
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        if blacklist.iter().map(|x| name.starts_with(x)).fold(false, |acc, i| acc || i) {
            continue;
        }

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // println!("adding file {:?} as {:?} ...", path, name);
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // println!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn compress_folder(src_dir: &str, dst_file: &str, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

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

