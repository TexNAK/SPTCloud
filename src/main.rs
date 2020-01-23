#![feature(seek_convenience)]
use std::error;
use clap::{Arg, App, SubCommand, ArgMatches};

mod client;
mod server;
mod logger;

use logger::Logger;
use log::{info, warn, error, Level};

fn main() {
    Logger::init(Level::Info);
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let matches = App::new("SPTCloud")
        .version("0.1")
        .author("Til Blechschmidt <til@blechschmidt.dev>")
        .about("Builds an SPT project on a remote server")
        .arg(Arg::with_name("host")
            .long("host")
            .value_name("address")
            .takes_value(true)
            .default_value("blechschmidt.de:2399")
            .help("Set the destination IP address of the request"))
        
        .arg(Arg::with_name("project-dir")
            .short("p")
            .long("project-dir")
            .value_name("path")
            .takes_value(true)
            .default_value("./")
            .help("Path to the project directory"))

        .arg(Arg::with_name("output-dir")
            .short("o")
            .long("output-dir")
            .value_name("path")
            .takes_value(true)
            .default_value("./out")
            .help("Path to the output directory"))

        .subcommand(SubCommand::with_name("server")
            .about("Hosts an SPTCloud server")
            .version("0.1")
            .author("Til Blechschmidt <til@blechschmidt.dev>")
            .arg(Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("address")
                .takes_value(true)
                .default_value("0.0.0.0:2399")
                .help("Set the local IP address the server binds to")))

        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        match run_server(&matches) {
            Ok(_) => warn!("Server stopped listening"),
            Err(e) => error!("{}", e),
        }
    } else {
        match run_client(&matches) {
            Ok(_) => info!("Project successfully built!"),
            Err(e) => error!("{}", e),
        }
    }

    return 0;
}

fn run_server(matches: &ArgMatches) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let bind_address = matches.value_of("bind").ok_or("Bind address not set")?;
    
    server::host(&bind_address)?;

    Ok(())
}

fn run_client(matches: &ArgMatches) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let source_folder = matches.value_of("project-dir").ok_or("Project directory not set")?;
    let output_folder = matches.value_of("output-dir").ok_or("Output directory not set")?;
    let host = matches.value_of("host").ok_or("Host not set")?;

    client::request_compilation(source_folder, output_folder, host)?;

    Ok(())
}