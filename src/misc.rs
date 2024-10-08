use anyhow::Result;
use once_cell::sync::Lazy;
use regex::{Captures, Regex, RegexBuilder};

pub const INDENT: &str = r#"(:?    |\t)"#;

const REQUIRED_MODULES_REGEX: &str = concat! {
    r#"(\["requiredModules"\] = \n)"#,
    r#"(\s+\{\n)"#,
    r#"(?:.+\n)+"#, // Do not capture any lines in the middle
    r#"(\s+\}, -- end of \["requiredModules"\]\n)"#,
};

pub fn remove_required_modules(mission: &str, dry_run: bool) -> Result<String> {
    static REGEX: Lazy<Regex> = Lazy::new(|| {
        RegexBuilder::new(REQUIRED_MODULES_REGEX)
            .multi_line(true)
            .build()
            .unwrap()
    });

    if !dry_run && !REGEX.is_match(mission) {
        println!(
            "?> The mission does not seem to have a requiredModules table, no need to remove it..."
        );
        return Ok(mission.to_owned());
    }

    Ok(REGEX
        .replace(mission, |cap: &Captures| {
            format!("{}{}{}", &cap[1], &cap[2], &cap[3])
        })
        .into_owned())
}
