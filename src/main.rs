use std::time::Instant;

use clap::Parser;

use crate::commands::Commands;

mod commands;
mod data;

fn main() {
    let commands = Commands::parse();
    let now = Instant::now();

    commands.handle_commands();

    println!("processing finished after {} ms", now.elapsed().as_millis());
}
