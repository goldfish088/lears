#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use std::boxed::Box;
use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

/*

pngme encode ./dice.png ruSt "This is a secret message!

pngme decode ./dice.png ruSt

pngme remove ./dice.png ruSt

pngme print ./dice.png

*/

#[derive(Parser)]
#[command(name = "pngme")]
#[command(about = "A PNG chunk secret message encoder/decoder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Encode {
        file_path: PathBuf,
        chunk_type: String,
        message: String,
        output_file: Option<PathBuf>,
    },
    Decode {
        file_path: PathBuf,
        chunk_type: String,
    },
    Remove {
        file_path: PathBuf,
        chunk_type: String,
    },
    Print {
        file_path: PathBuf,
    },
}

use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::Png;
use std::fs::File;
use std::io;
use std::str::FromStr;

fn open_png(file_path: &PathBuf) -> Result<Png> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    Png::try_from(contents.as_slice())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode {
            file_path,
            chunk_type,
            message,
            output_file,
        } => {
            let mut png = open_png(file_path)?;
            let chunk_type = ChunkType::from_str(chunk_type)?;
            let chunk = Chunk::new(chunk_type, Vec::from(message.clone()));
            png.append_chunk(chunk);
        }
        Commands::Decode {
            file_path,
            chunk_type,
        } => {
            let png = open_png(file_path)?;
            if let Some(chunk) = png.chunk_by_type(chunk_type) {
                println!("Found chunk: {}", chunk);
            } else {
                println!("Could not find such a chunk");
            }
        }
        Commands::Remove {
            file_path,
            chunk_type,
        } => {
            let mut png = open_png(file_path)?;
            if let Ok(chunk) = png.remove_first_chunk(chunk_type) {
                println!("Removed chunk: {}", chunk);
            } else {
                println!("Could not find such a chunk");
            }
        }
        Commands::Print { file_path } => {
            let png = open_png(file_path)?;
            println!("Marshalled PNG");
            println!("{}", png);
        }
    }

    Ok(())
}
