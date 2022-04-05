use crate::config::Weather;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::borrow::Cow;

fn modify_cloud_preset<'a>(mission: &'a str, weather: &Weather) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\["preset"\]) = ".+","#).unwrap());
    if !REGEX.is_match(mission) {
        return Err(anyhow!("Could not find cloud preset in mission file"));
    }
    println!("   Cloud preset: {}", weather.cloud_preset);
    Ok(REGEX.replace(mission, |cap: &Captures| {
        format!("{} = \"{}\",", &cap[1], weather.cloud_preset)
    }))
}

fn modify_cloud_base<'a>(mission: &'a str, weather: &Weather) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\["base"\]) = \d+,"#).unwrap());
    if let Some(cloud_base) = weather.random_cloud_base() {
        if !REGEX.is_match(mission) {
            return Err(anyhow!("Could not find cloud base key in mission file"));
        }
        println!("   Cloud base:   {}", cloud_base);
        Ok(REGEX.replace(mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], cloud_base)
        }))
    } else {
        Ok(Cow::Borrowed(mission))
    }
}

fn modify_temp<'a>(mission: &'a str, weather: &Weather) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"(\["temperature"\]) = [\d\.]+,"#).unwrap());
    if let Some(temperature) = weather.random_temp() {
        if !REGEX.is_match(mission) {
            return Err(anyhow!("Could not find temperature key in mission file"));
        }
        println!("   Temperature:  {:.2}", temperature);
        Ok(REGEX.replace(mission, |cap: &Captures| {
            format!("{} = {:.2},", &cap[1], temperature)
        }))
    } else {
        Ok(Cow::Borrowed(mission))
    }
}

fn modify_qnh<'a>(mission: &'a str, weather: &Weather) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\["qnh"\]) = [\d\.]+,"#).unwrap());
    if let Some(qnh) = weather.random_qnh() {
        if !REGEX.is_match(mission) {
            return Err(anyhow!("Could not find QNH key in mission file"));
        }
        println!("   QNH:          {:.2}", qnh);
        Ok(REGEX.replace(mission, |cap: &Captures| format!("{} = {:.2},", &cap[1], qnh)))
    } else {
        Ok(Cow::Borrowed(mission))
    }
}

pub fn modify_weather(mission: String, weather: &Weather) -> Result<String> {
    let mission = modify_cloud_preset(&mission, weather)?;
    let mission = modify_cloud_base(&mission, weather)?;
    let mission = modify_temp(&mission, weather)?;
    let mission = modify_qnh(&mission, weather)?;
    Ok(mission.into_owned())
}
