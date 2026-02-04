mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use std::boxed::Box;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    println!("Hello, world!");
}
