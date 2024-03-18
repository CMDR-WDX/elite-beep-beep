mod default_dir;
pub mod history;

use anyhow::anyhow;
use colored::Colorize;
use itertools::Itertools;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use rodio::source::SineWave;
use rodio::{OutputStream, Sink, Source};

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, read_dir};
use std::sync::{Arc, Mutex};
use std::{path::PathBuf, str::FromStr};
use std::{thread, time::Duration};

use clap::Parser;
use default_dir::try_get_default_directory;

use crate::history::{filter_for_only_relevant_entries, serialize_file_contents, MetInteraction};

#[derive(Parser)]
struct Cli {
    #[arg()]
    /// Path to game's CommanderHistory dir. Usually at %LOCALAPPDATA%/Frontier Developments/Elite Dangerous/CommanderHistory.  
    /// This path should point to the directory. This is optional, elite-beep-beep will try to make an educated guess.
    directory_path: Option<String>,
}

pub fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let (_stream, audio_device) = OutputStream::try_default().unwrap();
    let audio_sink = Sink::try_new(&audio_device).unwrap();
    audio_sink.set_volume(0.1);

    play_beep(&audio_sink);

    let state: Arc<Mutex<HashMap<OsString, Vec<MetInteraction>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let dir_path = match cli.directory_path {
        Some(val) => PathBuf::from_str(&val).map_err(|err| anyhow!(err)),
        None => try_get_default_directory(),
    }?;

    println!("Set up to listen at {:#?}", dir_path);

    let state_ref = state.clone();
    let handle_file_update = move |path: &PathBuf| {
        let file_contents = fs::read_to_string(path)?;
        eprintln!("{} - {:#?}", file_contents.len(), &path);
        let new_state = serialize_file_contents(&file_contents)?;
        let new_state = filter_for_only_relevant_entries(new_state);
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("Cannot extract file name."))?
            .to_os_string();

        // if file does not have a context assigned yet, simply assign without emitting a beep.
        match state_ref.lock().unwrap().entry(file_name) {
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(new_state);
            }
            std::collections::hash_map::Entry::Occupied(mut o) => {
                let previous_state = o.get();

                if previous_state != &new_state {
                    // no state change -> no need to beep
                    // Now get a list of CMDR IDs that are present in the new list, but missing in the old list.
                    let mut has_new_cmdrs = false;
                    {
                        let old_cmdrs_hashset: HashSet<u64> =
                            HashSet::from_iter(previous_state.into_iter().map(|x| x.commander_id));

                        for entry in &new_state {
                            if !old_cmdrs_hashset.contains(&entry.commander_id) {
                                has_new_cmdrs = true;
                                break;
                            }
                        }
                    }

                    // Set the updated state
                    o.insert(new_state);

                    // Play a beep if new CMDRs are present
                    if has_new_cmdrs {
                        play_beep(&audio_sink);
                    }
                    eprintln!(" --- ")
                }
            }
        };

        Ok::<(), anyhow::Error>(())
    };

    // Do a synthetic event for each file in dir to get the "initial state" going
    {
        let entries = read_dir(&dir_path)?;
        entries
            .into_iter()
            .filter_map(|e| match e {
                Ok(e) => match e.metadata() {
                    Err(_) => None,
                    Ok(metadata) => {
                        if !metadata.is_file() {
                            None
                        } else {
                            Some(e.path())
                        }
                    }
                },
                Err(_) => None,
            })
            .for_each(|path| match handle_file_update(&path) {
                Ok(_) => {
                    println!("Successfully handled {:#?}", &path.file_name().unwrap())
                }
                Err(err) => eprintln!("Error handling {:#?}: {}", &path.file_name().unwrap(), err),
            })
    }

    let mut watcher_debouncer = new_debouncer(
        Duration::from_millis(50),
        move |res: DebounceEventResult| match res {
            Err(err) => {
                eprintln!("watch error: {:?}", err)
            }
            Ok(events) => {
                eprintln!(
                    "{} - Received event count {}",
                    "DEBUG".yellow(),
                    events.len()
                );
                let _ = &events
                    .iter()
                    .for_each(|x| eprintln!("> {} - {:#?}", "DEBUG".yellow(), x));

                let paths: Vec<_> = events
                    .iter()
                    .filter(|x| match x.kind {
                        notify_debouncer_mini::DebouncedEventKind::AnyContinuous => true,
                        _ => false,
                    })
                    .map(|x| x.path.clone())
                    .unique()
                    .collect();
                if (&paths).is_empty() {
                    return;
                }

                for path in paths {
                    match handle_file_update(&path) {
                        Ok(_) => {}
                        Err(err) => eprintln!("Error when handling file: {}", err),
                    };
                }
            }
        },
    )
    .unwrap();

    watcher_debouncer
        .watcher()
        .watch(&dir_path, notify::RecursiveMode::NonRecursive)?;

    // Parking main thread until its cancelled via Ctrl-C
    thread::park();
    Ok(())
}

fn play_beep(audio_sink: &Sink) {
    println!("beep beep");
    audio_sink.append(SineWave::new(800.0).take_duration(Duration::from_millis(100)));
    audio_sink.append(SineWave::new(1600.0).take_duration(Duration::from_millis(200)));
}
