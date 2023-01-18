use crate::config::Preset;
use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex, RegexBuilder};

pub fn modify_time(mission: &str, preset: &Preset, dry_run: bool) -> Result<String> {
    // Note: we HAVE to replace the entry that is indented by exactly 4 spaces, because
    // there are other keys named "start_time" which we DON'T want to replace.
    static REGEX: Lazy<Regex> = Lazy::new(|| {
        RegexBuilder::new(r#"^(    \["start_time"\]) = \d+,$"#)
            .multi_line(true)
            .build()
            .unwrap()
    });

    let mut time = preset.time.splitn(3, ':');
    if true {
        let mut hours: i32 = time
            .next()
            .map(str::parse)
            .ok_or_else(|| anyhow!("Time is empty"))?
            .context(format!("cannot read hours from time {}", preset.time))?;
        let mut minutes: i32 = time
            .next()
            .map(str::parse)
            .unwrap_or(Ok(0))
            .context(format!("cannot read minutes from time {}", preset.time))?;
        let mut seconds: i32 = time
            .next()
            .map(str::parse)
            .unwrap_or(Ok(0))
            .context(format!("cannot read seconds from time {}", preset.time))?;

        // Normalize the time value by allowing overflow
        minutes += seconds / 60;
        seconds %= 60;
        hours += minutes / 60;
        minutes %= 60;
        hours %= 24;
        if !dry_run && !REGEX.is_match(mission) {
            return Err(anyhow!("Could not find start_time key in mission file"));
        }

        println!("   Start time: {:02}:{:02}:{:02}", hours, minutes, seconds);
        Ok(REGEX
            .replace(mission, |cap: &Captures| {
                // And de-normalize it back into seconds
                format!("{} = {},", &cap[1], hours * 3600 + minutes * 60 + seconds)
            })
            .into_owned())
    } else {
        Err(anyhow!("Invalid time format: {}", preset.time))
    }
}
