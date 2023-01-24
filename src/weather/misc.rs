use crate::config::Weather;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::borrow::Cow;

pub fn modify_temp<'a>(mission: &'a str, weather: &Weather, dry_run: bool) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"(\["temperature"\]) = [\d\.]+,"#).unwrap());

    if let Some(temperature) = weather.random_temp() {
        if !dry_run && !REGEX.is_match(mission) {
            return Err(anyhow!("Could not find temperature key in mission file"));
        }
        println!("   Temperature:           {:.2} °C", temperature);
        Ok(REGEX.replace(mission, |cap: &Captures| {
            format!("{} = {:.2},", &cap[1], temperature)
        }))
    } else {
        Ok(Cow::Borrowed(mission))
    }
}

pub fn modify_qnh<'a>(mission: &'a str, weather: &Weather, dry_run: bool) -> Result<Cow<'a, str>> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\["qnh"\]) = [\d\.]+,"#).unwrap());

    if let Some(qnh) = weather.random_qnh() {
        if !dry_run && !REGEX.is_match(mission) {
            return Err(anyhow!("Could not find QNH key in mission file"));
        }
        println!("   QNH:                   {:.2} mmHg", qnh);
        Ok(REGEX.replace(mission, |cap: &Captures| {
            format!("{} = {:.2},", &cap[1], qnh)
        }))
    } else {
        Ok(Cow::Borrowed(mission))
    }
}
