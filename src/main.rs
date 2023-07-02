mod config;
mod misc;
mod time;
mod weather;

use crate::{misc::remove_required_modules, time::modify_time, weather::modify_weather};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use config::{read_config, Config};
use crossterm::{
    event::{self, Event},
    terminal,
    tty::IsTty,
};
use once_cell::sync::Lazy;
use rand::{seq::SliceRandom, thread_rng};
use regex::{Captures, Regex};
use std::{
    collections::HashSet,
    env::{current_exe, set_current_dir},
    fs::File,
    io::{self, stdout, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::exit,
    thread::{sleep, spawn},
    time::Duration,
};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

pub fn flip_heading(heading: i32) -> i32 {
    (heading + 180) % 360
}

fn splice_filename(path: &str, new_suffix: &str, dry_run: bool) -> Result<String> {
    if dry_run {
        return Ok(String::from(path));
    }

    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.+)\.(.+)$").unwrap());
    if REGEX.is_match(path) {
        Ok(REGEX
            .replace(path, |caps: &Captures| {
                format!("{}_{}.{}", &caps[1], new_suffix, &caps[2])
            })
            .into_owned())
    } else {
        Err(anyhow!("File extension missing"))
    }
}

fn add_file(
    zip: &mut ZipWriter<File>,
    path: &str,
    data: &mut dyn Read,
    added_files: &mut HashSet<String>,
) -> Result<()> {
    let path = &path.replace('\\', "/");
    if !added_files.contains(path) {
        zip.start_file(path, FileOptions::default().compression_level(Some(9)))?;
        added_files.insert(path.to_owned());
        io::copy(data, zip)?;
    }
    Ok(())
}

fn add_repack_files(zip: &mut ZipWriter<File>, added_files: &mut HashSet<String>) -> Result<()> {
    let path = Path::new("repack");
    if path.is_dir() {
        println!("-> Repacking files from repack directory");
        for entry in WalkDir::new(path) {
            let entry = entry?;
            let fs_path = entry.path();
            if fs_path.is_file() {
                let zip_path = entry.path().strip_prefix("repack")?;
                let zip_path_str = zip_path
                    .to_str()
                    .ok_or_else(|| anyhow!("Cannot repack non UTF-8 path: {}", path.display()))?;
                add_file(
                    zip,
                    zip_path_str,
                    &mut File::open(fs_path)
                        .context(format!("Trying to open '{}' as file", fs_path.display()))?,
                    added_files,
                )?;
                println!("   Repacked {}", zip_path_str);
            }
        }
    }
    Ok(())
}

fn repack_miz(path: &str, mut config: Config, dry_run: bool) -> Result<()> {
    println!("Processing {path}...");
    let mut mission = String::new();
    let mut archive;
    let rng = &mut thread_rng();

    if dry_run {
        archive = None;
    } else {
        archive = Some(ZipArchive::new(File::open(path)?)?);
        archive
            .as_mut()
            .unwrap()
            .by_name("mission")?
            .read_to_string(&mut mission)?;
    }

    if config.misc.remove_required_modules {
        mission = remove_required_modules(&mission, dry_run)?;
    }

    for (name, preset) in &config.preset {
        let new_path = splice_filename(path, name, dry_run)?;
        let mut out_mission = mission.clone();
        println!("-> Generating miz preset: {name}");
        out_mission = modify_time(&out_mission, preset, dry_run)?;

        // Optionally, modify weather settings in the mission
        if let Some(weather_presets) = &preset.weather {
            for preset_name in weather_presets {
                if config.weather.get(preset_name).is_none() {
                    return Err(anyhow!("Weather preset not found: {preset_name}"));
                }
            }

            let preset_name = weather_presets
                .choose_weighted(rng, |weather| config.weather.get(weather).unwrap().weight)?;

            println!("-> Using weather preset:  {preset_name}");
            let weather = config.weather.get_mut(preset_name).unwrap();

            weather.randomize_wind_flip();
            if preset.flip_wind {
                weather.is_wind_flipped = !weather.is_wind_flipped
            }

            out_mission = modify_weather(out_mission, preset_name, weather, dry_run)?;
        }

        if !dry_run {
            println!("-> Writing new miz: {new_path}");
            let mut zip = ZipWriter::new(File::create(&new_path)?);
            let mut added_files = HashSet::new();
            let archive = archive.as_mut().unwrap();

            // Copy the modified mission file
            add_file(
                &mut zip,
                "mission",
                &mut out_mission.as_bytes(),
                &mut added_files,
            )?;

            // Copy files from the repack dir
            add_repack_files(&mut zip, &mut added_files)?;

            // Copy remaining miz files into the new zip
            for idx in 0..archive.len() {
                let mut file = archive.by_index(idx)?;
                let path = file.name().to_owned();
                add_file(&mut zip, &path, &mut file, &mut added_files)?;
            }

            zip.finish()?;
        }
        println!("-> Done\n");
    }

    if !dry_run {
        println!("Writing current path to \"most recently accessed\" file...");
        let mut recent_file = File::create(recent_file_path()?)?;
        write!(recent_file, "{}", Path::new(path).canonicalize()?.display())?;
        recent_file.flush()?;
    }

    println!("All done!\n");
    Ok(())
}

fn recent_file_path() -> Result<PathBuf> {
    current_exe()?
        .parent()
        .ok_or_else(|| anyhow!("Cannot find parent folder of exe"))
        .map(|path| path.join("repacker_recent.txt"))
}

fn miz_not_found_error<T>() -> Result<T> {
    Err(anyhow!(concat!(
        ".miz file not provided and no recent file was found\n",
        "Drag and drop a .miz file into the exe to run it"
    )))
}

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Optional: relative or absolute path of the DCS mission file to repack
    ///
    /// If the path is missing, the last repacked miz, as recorded in `repacker_recent.txt`, is used instead
    miz_path: Option<String>,

    /// Run and then exit immediately, without waiting for user input at the end
    #[clap(long, short)]
    batch: bool,

    /// Run without actually reading or writing any miz file
    #[clap(long, short)]
    dry_run: bool,
}

fn run(miz_path: &Option<String>, dry_run: bool) -> Result<()> {
    let config = read_config().context("Failed to read configuration from repack.toml")?;

    if dry_run {
        return repack_miz("dry run", config, true);
    }

    // Open either the argument or the most recently opened miz
    let miz_path = miz_path
        .as_ref()
        .map(|miz_path| Ok(miz_path.clone()))
        .unwrap_or_else(|| -> Result<String> {
            let recent_path = recent_file_path()?;
            if !recent_path.is_file() {
                return miz_not_found_error();
            }
            match BufReader::new(File::open(recent_path)?).lines().next() {
                Some(Ok(recent)) => {
                    println!("Trying most recently opened .miz: {recent}");
                    Ok(recent)
                }
                _ => miz_not_found_error(),
            }
        })?;

    // Switch to the miz directory
    set_current_dir(
        Path::new(&miz_path)
            .canonicalize()
            .with_context(|| format!("Cannot open {miz_path}"))?
            .parent()
            .ok_or_else(|| anyhow!("Cannot find parent folder of {miz_path}"))?,
    )?;

    repack_miz(&miz_path, config, false).with_context(|| format!("Failed to process {miz_path}"))
}

fn pause_and_exit(code: i32, batch: bool) -> ! {
    // Exit if not running in a terminal or in non-interactive mode
    if !stdout().is_tty() || batch {
        exit(code);
    }
    // Auto-exit if the user doesn't respnd
    spawn(move || {
        sleep(Duration::from_secs(30));
        eprintln!("Timed out waiting for response");
        exit(code);
    });
    // Wait for user response...
    eprintln!("Press any key or wait 30 seconds to continue...");
    terminal::enable_raw_mode().unwrap();
    loop {
        if let Event::Key(_) = event::read().unwrap() {
            exit(code);
        }
    }
}

fn main() {
    match Args::try_parse() {
        Ok(args) => match run(&args.miz_path, args.dry_run) {
            Ok(_) => pause_and_exit(0, args.batch),
            Err(err) => {
                eprintln!("{err:?}\n");
                pause_and_exit(1, args.batch);
            }
        },
        Err(err) if err.use_stderr() => {
            err.print().unwrap();
            eprintln!();
            pause_and_exit(2, false);
        }
        Err(err) => err.exit(),
    }
}
