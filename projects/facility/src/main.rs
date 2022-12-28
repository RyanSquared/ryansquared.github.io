use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use color_eyre::{Result, Section};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config file could not be read: {0}")]
    Io(#[from] std::io::Error),

    #[error("config could not be deserialized: {0}")]
    Json(#[from] serde_json::Error),
}

fn load_config_from_file(config_file: impl AsRef<Path>) -> Result<Value, Error> {
    let file = File::open(config_file)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let config = load_config_from_file("config.json")
        .suggestion("try copying the example config: `cp config.example.json config.json`")?;

    println!("config: {config:?}");

    Ok(())
}
