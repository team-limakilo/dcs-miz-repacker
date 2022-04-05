use anyhow::Result;
use rand::{thread_rng, Rng};
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
    pub cloud_base_min: Option<i32>,
    pub cloud_base_max: Option<i32>,
    pub temp_min: Option<f64>,
    pub temp_max: Option<f64>,
    pub qnh_min: Option<f64>,
    pub qnh_max: Option<f64>,
    pub weight: f64,
}

impl Weather {
    pub fn random_cloud_base(&self) -> Option<i32> {
        match (self.cloud_base_min, self.cloud_base_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        }
    }
    pub fn random_temp(&self) -> Option<f64> {
        match (self.temp_min, self.temp_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        }
    }
    pub fn random_qnh(&self) -> Option<f64> {
        match (self.qnh_min, self.qnh_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        }
    }
}

pub fn read_config() -> Result<Config> {
    let mut data = Vec::new();
    File::open("repack.toml")?.read_to_end(&mut data)?;
    Ok(toml::from_slice(&data)?)
}
