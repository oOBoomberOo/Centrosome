#[macro_use]
extern crate clap;

use clap::App;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

use std::fs::{remove_dir_all, DirEntry};
use std::path::Path;
use std::thread;

mod zip_peaker;

fn main() {
	let yaml = load_yaml!("../resource/cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let directory = matches.value_of("directory").unwrap();
	let directory = Path::new(directory);

	if directory.exists() {
		if directory.is_dir() {
			merge(&directory);
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

fn merge(directory: &Path) {
	let temp_dir = std::env::temp_dir().join("datapack-merger");

	// Return all zip files in `directory`
	let result: Vec<DirEntry> = directory
		.read_dir()
		.unwrap()
		.filter_map(|entry| entry.ok())
		// .filter(|entry: &DirEntry| entry.path().is_file() && entry.path().ends_with(".zip"))
		.collect();

	let selection_items: Vec<String> = result
		.iter()
		.map(|entry: &DirEntry| entry.file_name().to_str().unwrap().to_string())
		.collect();

	let _selection = Select::with_theme(&ColorfulTheme::default())
		.with_prompt("Please choose core datapack")
		.default(0)
		.items(&selection_items)
		.interact()
		.unwrap();

	let progress_bars = MultiProgress::new();

	// Map all "datapack names" to numerical value
	let mut name_lengths: Vec<usize> = result
		.iter()
		.map(|entry| entry.file_name().to_str().unwrap().to_string().len())
		.collect();
	// Sort it to quickly get the highest value
	name_lengths.sort();
	let max_length = name_lengths.last().unwrap();

	// Create progress bars *before* running .par_iter() because that's when thread blocking happen.
	let result_with_progress_bars: Vec<(DirEntry, ProgressBar)> = result
		.into_iter()
		.map(|entry| {
			let progress_bar = create_progress_bar(&progress_bars, &entry, *max_length);
			(entry, progress_bar)
		})
		.collect();

	// MultiProgress have to be run in another thread so that .par_iter() won't block each other process.
	thread::spawn(move || {
		progress_bars.join().unwrap();
	});

	result_with_progress_bars
		.par_iter()
		// .iter()
		.for_each(|(entry, progress_bar)| {
			zip_peaker::peak(entry, &temp_dir, &progress_bar).unwrap();
		});

	remove_dir_all(&temp_dir).unwrap();
}

fn create_progress_bar(
	multi_progress: &MultiProgress,
	entry: &DirEntry,
	width: usize,
) -> ProgressBar {
	let entry_name = entry.file_name().to_str().unwrap().bright_green().bold();
	let template = format!("[{{elapsed}}] {:<width$} [{{wide_bar:.cyan/white}}] {{pos:>4}}/{{len:4}} {{percent:>3}}% ({{eta:^4}})", entry_name, width = width);
	let progress_bar = ProgressBar::new(1000).with_style(
		ProgressStyle::default_bar()
			.template(&template)
			.progress_chars("#>-"),
	);
	multi_progress.add(progress_bar)
}
