use crate::config::Weather;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::borrow::Cow;

macro_rules! wind_regex {
    ($altitude:literal, $attribute:literal) => {
        Lazy::new(|| {
            let regex_str = concat!(
                r#"(\s{12}\["#,
                $altitude,
                r#"\] =.+"#,
                r#"\s{16}\["#,
                $attribute,
                r#"]) = \d+,"#
            );
            Regex::new(&regex_str).unwrap()
        })
    };
}

pub fn modify_ground_wind<'a>(
    mission: &'a str,
    weather: &Weather,
    wind_ground_speed: &mut f64,
    dry_run: bool,
) -> Result<Cow<'a, str>> {
    static SPEED_REGEX: Lazy<Regex> = wind_regex!("atGround", "speed");
    static HEADING_REGEX: Lazy<Regex> = wind_regex!("atGround", "dir");

    let mut mission = Cow::Borrowed(mission);

    if let Some(wind_speed) = weather.random_wind_speed_ground() {
        if !dry_run && !SPEED_REGEX.is_match(&mission) {
            return Err(anyhow!(
                "Could not find ground wind speed key in mission file"
            ));
        }
        println!("   Ground wind speed:     {:.1} m/s", wind_speed);
        let new_mission = SPEED_REGEX.replace(&mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], wind_speed)
        });

        *wind_ground_speed = wind_speed;
        mission = Cow::Owned(new_mission.into_owned());
    }

    if let Some(wind_heading) = weather.random_wind_heading_ground() {
        if !dry_run && !HEADING_REGEX.is_match(&mission) {
            return Err(anyhow!(
                "Could not find ground wind direction key in mission file"
            ));
        }
        println!("   Ground wind heading:   {}°", wind_heading);
        let new_mission = HEADING_REGEX.replace(&mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], wind_heading)
        });

        mission = Cow::Owned(new_mission.into_owned());
    }

    Ok(mission)
}

pub fn modify_2000m_wind<'a>(
    mission: &'a str,
    weather: &Weather,
    ground_speed: f64,
    wind_2000m_speed: &mut f64,
    dry_run: bool,
) -> Result<Cow<'a, str>> {
    static SPEED_REGEX: Lazy<Regex> = wind_regex!("at2000m", "speed");
    static HEADING_REGEX: Lazy<Regex> = wind_regex!("at2000m", "dir");

    let mut mission = Cow::Borrowed(mission);

    if let Some(wind_speed) = weather.random_wind_speed_2000m(ground_speed) {
        if !dry_run && !SPEED_REGEX.is_match(&mission) {
            return Err(anyhow!(
                "Could not find 2000m wind speed key in mission file"
            ));
        }
        println!("   2000m wind speed:      {:.1} m/s", wind_speed);
        let new_mission = SPEED_REGEX.replace(&mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], wind_speed)
        });

        *wind_2000m_speed = wind_speed;
        mission = Cow::Owned(new_mission.into_owned());
    }

    if let Some(wind_heading) = weather.random_wind_heading_2000m() {
        if !dry_run && !HEADING_REGEX.is_match(&mission) {
            return Err(anyhow!(
                "Could not find 2000m wind direction key in mission file"
            ));
        }
        println!("   2000m wind heading:    {}°", wind_heading);
        let new_mission = HEADING_REGEX.replace(&mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], wind_heading)
        });

        mission = Cow::Owned(new_mission.into_owned());
    }

    Ok(mission)
}

pub fn modify_8000m_wind<'a>(
    mission: &'a str,
    weather: &Weather,
    wind_2000m_speed: f64,
    dry_run: bool,
) -> Result<Cow<'a, str>> {
    static SPEED_REGEX: Lazy<Regex> = wind_regex!("at8000m", "speed");
    static HEADING_REGEX: Lazy<Regex> = wind_regex!("at8000m", "dir");

    let mut mission = Cow::Borrowed(mission);

    if let Some(wind_speed) = weather.random_wind_speed_8000m(wind_2000m_speed) {
        if !dry_run && !SPEED_REGEX.is_match(&mission) {
            return Err(anyhow!(
                "Could not find 8000m wind speed key in mission file"
            ));
        }
        println!("   8000m wind speed:      {:.1} m/s", wind_speed);
        let new_mission = SPEED_REGEX.replace(&mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], wind_speed)
        });

        mission = Cow::Owned(new_mission.into_owned());
    }

    if let Some(wind_heading) = weather.random_wind_heading_8000m() {
        if !dry_run && !HEADING_REGEX.is_match(&mission) {
            return Err(anyhow!(
                "Could not find 8000m wind direction key in mission file"
            ));
        }
        println!("   8000m wind heading:    {}°", wind_heading);
        let new_mission = HEADING_REGEX.replace(&mission, |cap: &Captures| {
            format!("{} = {},", &cap[1], wind_heading)
        });

        mission = Cow::Owned(new_mission.into_owned());
    }

    Ok(mission)
}
