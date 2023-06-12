use clap::StructOpt;
mod commands;
mod args;
mod chunk_type;
mod chunk;
mod png;
use commands::execute_command;
use args::Cli;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;
fn main() {
    let args = Cli::parse();
    match execute_command(args.command) {
        Ok(()) => println!("Worked successfully."),
        Err(why) => println!("{}", why),
    }
}