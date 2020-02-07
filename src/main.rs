#[macro_use]
extern crate clap;

use clap::App;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};

use std::fs::{remove_dir_all, DirEntry};
use std::path::{Path, PathBuf};
use std::thread;

mod datapacks;
mod utils;
mod zipper;

use datapacks::Datapack;
use utils::{create_zipper, get_longest_name_length, read_directory, get_path_name, Result};
use zipper::Zipper;

fn main() {
	let yaml = load_yaml!("../resource/cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let directory = matches.value_of("directory").expect("Invalid directory name");
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

/**
 * Handling all the command interaction with user
 * 
 * The program work as follow:
 * 1) Read directory and identify datapack-like files
 * 2) Ask user to select 'core datapack'
 * 3) Get the longest name from all of the datapacks (will be used to padded the progress bar to be equal length)
 * 4) Create 'Zipper' for all datapack. In this step progress bar for "extracting" datapack will also be created as well
 * 5) Extract all zipped datapacks
 * 6) Interpreting all datapacks into 'Datapack' struct
 * 7) Merge every datapacks except 'core datapack'
 * 8) Merge 'core datapack'
 * 9) Remove all files inside temp directory used for extracting zip files
 * 10) Compress the merged datapack
 */
fn merge(directory: &Path) -> Result<()> {
	let temp_dir = std::env::temp_dir().join("datapack-merger");

	let setting_up_progress_bar = ProgressBar::new(100)
		.with_style(
			ProgressStyle::default_bar()
				.template("[{elapsed}] Setting up... [{wide_bar:.cyan/white}] {pos:.green}/{len:.white} {percent}%")
				.progress_chars("#>_")
		);

	// Return all zip files in `directory`
	let result: Vec<DirEntry> = read_directory(directory, setting_up_progress_bar)?;

	let selection_items: Vec<String> = result
		.par_iter()
		.map(|entry| get_path_name(&entry.path()))
		.collect();

	let selection = Select::with_theme(&ColorfulTheme::default())
		.with_prompt("Please choose core datapack")
		.default(0)
		.items(&selection_items)
		.interact()?;

	let merged_datapack_name = Input::<String>::with_theme(&ColorfulTheme::default())
		.with_prompt("Merged Datapack name")
		.default("merged_datapack".to_string())
		.allow_empty(false)
		.interact()?;

	let progress_bars = MultiProgress::new();

	let max_length = get_longest_name_length(selection_items.as_slice());

	// Create progress bars *before* running .par_iter() because that's when thread blocking happen.
	let zippers: Vec<Zipper> = result
		.into_iter()
		.filter_map(|entry| create_zipper(&entry, max_length, &progress_bars).ok())
		.collect();

	let mut threads = Vec::default();

	// MultiProgress have to be run in another thread so that .par_iter() won't block each other process.
	let progress_bar_thread = thread::spawn(move || {
		progress_bars.join().unwrap();
	});

	threads.push(progress_bar_thread);

	let datapacks: Vec<Datapack> = zippers
		.par_iter()
		.filter_map(|zipper| zipper.datapack(&temp_dir).ok())
		.collect();

	for process in threads {
		process.join().expect("panic in child thread");
	}
	
	println!("Finished interpreting {} datapacks.", datapacks.len());

	let merging_progress_bar = ProgressBar::new(datapacks.len() as u64).with_style(
		ProgressStyle::default_bar()
			.template(&format!(
				"[{{elapsed}}] {0} [{{wide_bar:.cyan/white}}] {{percent}}% {{msg}}",
				"Merging...".yellow().bold()
			))
			.progress_chars("#>_"),
	);

	let selection = selection_items[selection].to_owned();
	let core_datapack = datapacks
		.par_iter()
		.find_first(|datapack| datapack.name == selection)
		.expect("Unable to read core datapack")
		.to_owned();

	let datapack_dir = temp_dir.join(".merged-datapack");
	let mut new_datapack = Datapack::new(".merged-datapack", datapack_dir);

	datapacks
		.into_iter()
		.filter(|datapack| datapack.name != core_datapack.name)
		.for_each(|datapack| {
			new_datapack = new_datapack.merge(datapack);
			merging_progress_bar.inc(1);
		});

	let merged_datapack = new_datapack.merge(core_datapack);

	remove_dir_all(temp_dir)?;

	merging_progress_bar.finish_with_message("[Finished]");

	let merged_datapack_path = PathBuf::from(format!("{}.zip", merged_datapack_name));
	let output_path = directory.join(merged_datapack_path);

	let template = format!("[{{elapsed}}] {} [{{wide_bar:.cyan/white}}] {{percent:>3}}% {{msg}}", "Compiling...".yellow().bold());
	let compiling_progress_bar = ProgressBar::new(1)
		.with_style(ProgressStyle::default_bar()
			.template(&template)
			.progress_chars("#>_")
		);

	merged_datapack.compile(&output_path, compiling_progress_bar)?;

	println!("Output merged datapack to: '{}'", output_path.display().to_string().cyan());

	Ok(())
}