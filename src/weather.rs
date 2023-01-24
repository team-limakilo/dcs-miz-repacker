use crate::config::Weather;
use anyhow::Result;

mod clouds;
mod misc;
mod wind;

pub fn modify_weather(
    mission: String,
    preset_name: &str,
    weather: &Weather,
    dry_run: bool,
) -> Result<String> {
    let mut wind_ground_speed = 0.0;
    let mut wind_2000m_speed = 0.0;
    let mission = clouds::modify_cloud_preset(&mission, preset_name, weather, dry_run)?;
    let mission = clouds::modify_cloud_base(&mission, weather, dry_run)?;
    let mission = wind::modify_ground_wind(&mission, weather, &mut wind_ground_speed, dry_run)?;
    let mission = wind::modify_2000m_wind(
        &mission,
        weather,
        wind_ground_speed,
        &mut wind_2000m_speed,
        dry_run,
    )?;
    let mission = wind::modify_8000m_wind(&mission, weather, wind_2000m_speed, dry_run)?;
    let mission = misc::modify_temp(&mission, weather, dry_run)?;
    let mission = misc::modify_qnh(&mission, weather, dry_run)?;
    Ok(mission.into_owned())
}
