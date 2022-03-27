use anyhow::Result;
use serde_derive::Deserialize;
use std::{collections::HashMap, fs::File, io::Read};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub preset: HashMap<String, Preset>,
    pub weather: HashMap<String, Weather>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub weather: Option<Vec<String>>,
    pub time: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weather {
    pub cloud_preset: String,
    pub cloud_base_min: i32,
    pub cloud_base_max: i32,
    pub weight: f64,
}

pub fn read_config() -> Result<Config> {
    let mut data = Vec::new();
    File::open("repack.toml")?.read_to_end(&mut data)?;
    Ok(toml::from_slice(&data)?)
}
