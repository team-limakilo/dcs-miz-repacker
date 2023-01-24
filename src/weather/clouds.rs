use crate::config::Weather;
use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::borrow::Cow;

pub fn modify_cloud_preset<'a>(
    mission: &'a str,
    preset_name: &'a str,
    weather: &Weather,
    dry_run: bool,
) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\["preset"\]) = ".+","#).unwrap());

    if !dry_run && !REGEX.is_match(mission) {
        return Err(anyhow!("Could not find cloud preset in mission file"));
    }

    let cloud_preset = weather
        .cloud_preset
        .as_ref()
        .with_context(|| format!("Cloud preset not defined in weather key: {preset_name}"))?;

    println!("   Cloud preset:          {cloud_preset}",);
    Ok(REGEX.replace(mission, |cap: &Captures| {
        format!("{} = \"{}\",", &cap[1], cloud_preset)
    }))
}

pub fn modify_cloud_base<'a>(
    mission: &'a str,
    weather: &Weather,
    dry_run: bool,
) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\["base"\]) = [\d\.]+,"#).unwrap());

    if let Some(cloud_base) = weather.random_cloud_base() {
        if !dry_run && !REGEX.is_match(mission) {
            return Err(anyhow!("Could not find cloud base key in mission file"));
        }
        println!("   Cloud base:            {} meters", cloud_base);
        Ok(REGEX.replace(mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], cloud_base)
        }))
    } else {
        Ok(Cow::Borrowed(mission))
    }
}
