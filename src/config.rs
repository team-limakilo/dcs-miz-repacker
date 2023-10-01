use anyhow::{Context, Result};
use rand::{thread_rng, Rng};
use serde_derive::Deserialize;
use std::{collections::HashMap, fs::File, io::Read};
use toml::{value::Table, Value};

use crate::flip_heading;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub misc: MiscOptions,
    pub preset: HashMap<String, Preset>,
    pub weather: HashMap<String, Weather>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiscOptions {
    #[serde(default)]
    pub remove_required_modules: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub weather: Option<Vec<String>>,
    pub time: String,

    #[serde(default)]
    pub flip_wind: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weather {
    pub cloud_preset: Option<String>,
    pub cloud_base_min: Option<i32>,
    pub cloud_base_max: Option<i32>,

    pub wind_ground_speed_min: Option<f64>,
    pub wind_ground_speed_max: Option<f64>,
    pub wind_ground_heading_min: Option<i32>,
    pub wind_ground_heading_max: Option<i32>,
    pub wind_2000m_increase_speed_min: Option<f64>,
    pub wind_2000m_increase_speed_max: Option<f64>,
    pub wind_2000m_heading_min: Option<i32>,
    pub wind_2000m_heading_max: Option<i32>,
    pub wind_8000m_increase_speed_min: Option<f64>,
    pub wind_8000m_increase_speed_max: Option<f64>,
    pub wind_8000m_heading_min: Option<i32>,
    pub wind_8000m_heading_max: Option<i32>,

    #[serde(default)]
    pub wind_flip_chance: f64,

    pub temp_min: Option<f64>,
    pub temp_max: Option<f64>,

    pub qnh_min: Option<f64>,
    pub qnh_max: Option<f64>,

    #[serde(default = "default_weight")]
    pub weight: f64,

    #[serde(default)]
    pub inherit: Vec<String>,

    #[serde(skip_serializing, default)]
    pub is_wind_flipped: bool,
}

fn default_weight() -> f64 {
    1.0
}

// TODO: generate most of this with a macro
impl Weather {
    pub fn randomize_wind_flip(&mut self) {
        self.is_wind_flipped = thread_rng().gen_bool(self.wind_flip_chance);
    }
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
    pub fn random_wind_heading_ground(&self) -> Option<i32> {
        let mut hdg = match (self.wind_ground_heading_min, self.wind_ground_heading_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        };
        if self.is_wind_flipped {
            hdg = hdg.map(flip_heading);
        }
        hdg
    }
    pub fn random_wind_speed_2000m(&self, wind_ground_speed: f64) -> Option<f64> {
        match (
            self.wind_2000m_increase_speed_min,
            self.wind_2000m_increase_speed_max,
        ) {
            (None, None) => None,
            (None, max) => max.map(|x| wind_ground_speed + x),
            (min, None) => min.map(|x| wind_ground_speed + x),
            (Some(min), Some(max)) => Some(wind_ground_speed + thread_rng().gen_range(min..=max)),
        }
    }
    pub fn random_wind_heading_2000m(&self) -> Option<i32> {
        let mut hdg = match (self.wind_2000m_heading_min, self.wind_2000m_heading_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        };
        if self.is_wind_flipped {
            hdg = hdg.map(flip_heading);
        }
        hdg
    }
    pub fn random_wind_speed_8000m(&self, wind_2000m_speed: f64) -> Option<f64> {
        match (
            self.wind_8000m_increase_speed_min,
            self.wind_8000m_increase_speed_max,
        ) {
            (None, None) => None,
            (None, max) => max.map(|x| wind_2000m_speed + x),
            (min, None) => min.map(|x| wind_2000m_speed + x),
            (Some(min), Some(max)) => Some(wind_2000m_speed + thread_rng().gen_range(min..=max)),
        }
    }
    pub fn random_wind_heading_8000m(&self) -> Option<i32> {
        let mut hdg = match (self.wind_8000m_heading_min, self.wind_8000m_heading_max) {
            (None, None) => None,
            (None, max) => max,
            (min, None) => min,
            (Some(min), Some(max)) => Some(thread_rng().gen_range(min..=max)),
        };
        if self.is_wind_flipped {
            hdg = hdg.map(flip_heading);
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

fn preprocess_inheritance(mut config_data: Value) -> Result<Value> {
    if let Some(presets) = config_data
        .get_mut("weather")
        .and_then(|weather_data| weather_data.as_table_mut())
    {
        let mut inheritance_pairs: Vec<(String, String)> = Vec::new();

        // List out all presets and their inherited presets
        for (preset_name, preset) in presets.iter() {
            if let Some(inherits) = preset.get("inherit").and_then(|inherit| inherit.as_array()) {
                for inherit in inherits {
                    inheritance_pairs
                        .push((preset_name.clone(), inherit.as_str().unwrap().to_owned()));
                }
            }
        }

        // Replace them in the presets map
        for (preset_name, inherited_name) in inheritance_pairs {
            let preset = presets
                .get(&preset_name)
                .unwrap()
                .as_table()
                .unwrap()
                .clone();

            let inherited_values = presets.get(&inherited_name)
                .with_context(|| format!("Preset '{preset_name}' tries to inherit values from '{inherited_name}', but the referenced preset does not exist"))?
                .as_table().with_context(|| format!("Preset '{preset_name}' tries to inherit values from '{inherited_name}', but it is not a table"))?;

            presets.insert(
                preset_name,
                Value::Table(merge_tables(preset, inherited_values)),
            );
        }
    }

    Ok(config_data)
}

fn merge_tables(mut destination: Table, source: &Table) -> Table {
    for (key, value) in source {
        destination.insert(key.clone(), value.clone());
    }
    destination
}
