use anyhow::{Context, Result};
use rand::{thread_rng, Rng};
use serde_derive::Deserialize;
use std::{collections::HashMap, fs::File, io::Read};
use toml::{value::Table, Value};

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

    #[serde(default = "default_weight")]
    pub weight: f64,

    #[serde(default)]
    pub inherit: Vec<String>,
}

fn default_weight() -> f64 {
    1.0
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
    pub fn random_wind_speed_ground(&self) -> Option<f64> {
        match (self.wind_ground_speed_min, self.wind_ground_speed_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        }
    }
    pub fn random_wind_heading_ground(&self) -> Option<f64> {
        let mut hdg = match (self.wind_ground_heading_min, self.wind_ground_heading_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        };
        if self.is_wind_flipped {
            hdg = hdg.map(|hdg| (hdg + 180.0) % 360.0);
        }
        hdg
    }
}

pub fn read_config() -> Result<Config> {
    let mut data = Vec::new();

    File::open("repack.toml")
        .or_else(|original_error| File::open("example/repack.toml").map_err(|_| original_error))?
        .read_to_end(&mut data)?;

    let config_data = preprocess_inheritance(toml::from_slice(&data)?)?;
    Ok(config_data.try_into()?)
}

fn preprocess_inheritance(config_data: Value) -> Result<Value> {
    let mut new_data = config_data.clone();
    if let Some(presets) = config_data
        .get("weather")
        .and_then(|weather_data| weather_data.as_table())
    {
        for (preset_name, preset) in presets {
            if let Some(inherits) = preset.get("inherit").and_then(|inherit| inherit.as_array()) {
                for inherit in inherits {
                    if let Some(inherited_name) = inherit.as_str() {
                        let preset = preset.as_table().unwrap();
                        let inherited = presets.get(inherited_name)
                            .with_context(|| format!("Preset '{preset_name}' tries to inherit values from '{inherited_name}', but the referenced preset does not exist"))?
                            .as_table().with_context(|| format!("Preset '{preset_name}' tries to inherit values from '{inherited_name}', but it is not a table"))?;

                        let new_preset = merge_tables(inherited, preset);

                        new_data
                            .get_mut("weather")
                            .unwrap()
                            .as_table_mut()
                            .unwrap()
                            .insert(preset_name.clone(), new_preset.into());
                    }
                }
            }
        }
    }
    Ok(new_data)
}

fn merge_tables(destination: &Table, target: &Table) -> Table {
    let mut result = target.clone();
    for (key, value) in destination {
        result.insert(key.clone(), value.clone());
    }
    result
}
