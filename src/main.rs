#[macro_use]
extern crate clap;

use clap::App;
use colored::*;
use indicatif::MultiProgress;
use rayon::prelude::*;

use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

use std::fs::DirEntry;
use std::path::{Path};
use std::thread;
use std::io::{stdout, Write};

mod datapacks;
mod utils;
mod zipper;

use datapacks::{Datapack};
use utils::{create_zipper, entry_name, get_longest_name_length, read_directory, Result};
use zipper::Zipper;

fn main() {
	let yaml = load_yaml!("../resource/cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let directory = matches.value_of("directory").unwrap();
	let directory = Path::new(directory);

	if directory.exists() {
		if directory.is_dir() {
			if let Err(error) = merge(&directory) {
				eprintln!("{}", error);
			}
		} else {
			eprintln!(
				"'{}' is not a directory",
				format!("{}", directory.display()).cyan()
			);
		}
	} else {
		eprintln!(
			"'{}' does not exist.",
			format!("{}", directory.display()).cyan()
		);
	}
}

fn merge(directory: &Path) -> Result<()> {
	let temp_dir = std::env::temp_dir().join("datapack-merger");

	// Return all zip files in `directory`
	let result: Vec<DirEntry> = read_directory(directory)?;

	let selection_items: Vec<String> = result
		.iter()
		.map(entry_name)
		.filter_map(|entry| entry.ok())
		.collect();

	let _selection = Select::with_theme(&ColorfulTheme::default())
		.with_prompt("Please choose core datapack")
		.default(0)
		.items(&selection_items)
		.interact()?;

	let progress_bars = MultiProgress::new();

	let max_length = get_longest_name_length(selection_items.as_slice());

	// Flush first because loading bar seem to disappear when run in short enough time
	stdout().flush()?;

	// Create progress bars *before* running .par_iter() because that's when thread blocking happen.
	let zippers: Vec<Zipper> = result
		.into_iter()
		.filter_map(|entry| create_zipper(&entry, &temp_dir, max_length, &progress_bars).ok())
		.collect();

	let mut threads = Vec::default();

	// MultiProgress have to be run in another thread so that .par_iter() won't block each other process.
	let progress_bar_thread = thread::spawn(move || {
		progress_bars.join().unwrap();
	});

	threads.push(progress_bar_thread);

	let result: Vec<Datapack> = zippers.par_iter().filter_map(|zipper| zipper.datapack().ok()).collect();

	for process in threads {
		process.join().expect("panic in child thread");
	}

	println!("{:#?}", result);

	Ok(())
}
