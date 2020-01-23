#![feature(seek_convenience)]
use std::{env};

mod client;
mod server;

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let args: Vec<_> = env::args().collect();

    if args.len() == 2 && args[1] == "server" {
        match server::host() {
            Ok(_) => println!("server exited."),
            Err(e) => println!("Error: {:?}", e),
        }
    } else {
        match client::request_compilation() {
            Ok(_) => println!("done."),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    return 0;
}

