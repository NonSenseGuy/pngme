use std::borrow::Borrow;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::Path;

use colored::Colorize;

use crate::args::CliCommand;
use crate::chunk::{Chunk, self};
use crate::chunk_type::ChunkType;
use crate::png::Png;

const DEFAULT_CHUNK_TYPE: &str = "ruSt";

fn get_png(file_path: &String) -> Result<Png, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let mut file = File::open(path)?;

    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let png = Png::try_from(buf.as_ref())?;

    Ok(png)
}

fn overwrite_file(file_path: &String, buf: &[u8]) -> Result<(), Error> {
    let mut file = File::options()
        .truncate(true)
        .write(true)
        .open(Path::new(file_path))?;
    file.write_all(buf).unwrap();

    Ok(())
}

pub fn execute_command(command: CliCommand) -> Result<(), Box<dyn std::error::Error>> {
    use CliCommand::*;

    match command {
        Decode { file_path } => {
            let png = get_png(&file_path).unwrap();

            if let Some(chunk) = png.chunk_by_type(DEFAULT_CHUNK_TYPE) {
                println!("{}", chunk.data_as_string().unwrap());
            } else {
                println!("{} No message hidden in {file_path}", "Error:".red().bold())
            }
        }
        Encode {
            file_path,
            message,
            output_file,
        } => {
            let mut png =
                get_png(&file_path).unwrap_or_else(|_| panic!("File {} not found", &file_path));
        
            let chunk_type = ChunkType::try_from(<[u8;4]>::try_from(DEFAULT_CHUNK_TYPE.as_bytes())?).unwrap();
            print!("{}", chunk_type);
            let new_chunk : Chunk = Chunk::new(chunk_type, message.into_bytes());

            png.append_chunk(new_chunk);
            let buf = png.as_bytes();

            match output_file {
                None => overwrite_file(&file_path, buf.as_ref())?,
                Some(x) => overwrite_file(&x, buf.as_ref())?,
            }

            println!(
                "{} Wrote message to '{}'",
                "SUCCESS:".bright_green().bold(),
                file_path.blue(),
            );
        }
        Remove { file_path } => {
            let mut png = get_png(&file_path)?;
            png.remove_chunk(DEFAULT_CHUNK_TYPE).unwrap();
            overwrite_file(&file_path, png.as_bytes().as_ref()).unwrap();
        }
        Print { file_path } => {
            let png = get_png(&file_path)?;
            println!("{}", png);
        }
    };

    Ok(())
}