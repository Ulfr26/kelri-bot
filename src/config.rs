use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use serenity::model::id::ChannelId;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub channels: HashMap<ChannelId, ChannelId>,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let mut file = File::open(path.as_ref())?;
        let mut buf = String::new();

        file.read_to_string(&mut buf)?;

        Ok(Self {
            channels: serde_json::from_str(&buf)?,
        })
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let mut file = File::create(path.as_ref())?;

        println!("json: {}", serde_json::to_string_pretty(&self.channels)?);
        file.write_all(serde_json::to_string_pretty(&self.channels)?.as_bytes())?;

        Ok(())
    }
}
