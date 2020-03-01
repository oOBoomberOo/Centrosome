#[macro_use]
extern crate clap;

use clap::App;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};

use std::io;
use std::path::{Path, PathBuf};

mod datapack_loader;
mod datapacks;
mod utils;

use datapack_loader::DatapackLoader;
use datapacks::Datapack;
use utils::{
	get_compression_method, get_datapacks, os_str_to_string, DatapackIterator, MergeError,
};

fn main() {
	let yaml = load_yaml!("../resource/cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let directory = matches
		.value_of("directory")
		.expect("Invalid directory name");
	let directory = Path::new(directory);

	if directory.exists() {
		if directory.is_dir() {
			if let Err(error) = merge(&directory) {
				eprintln!("{}", error);
			}
		} else {
			eprintln!(
				"'{}' is not a directory!",
				directory.display().to_string().cyan()
			);
		}
	} else {
		eprintln!(
			"'{}' {}",
			directory.display().to_string().cyan(),
			"does not exists.".red()
		);
	}
}

fn merge(directory: &Path) -> Result<(), MergeError> {
	let datapack_entries = get_datapacks(directory)?;
	let (selection_items, datapack_entries) = get_selection_items(datapack_entries);

	let selection = match ask_core_datapack(&selection_items)? {
		Some(x) => x,
		None => return Err(MergeError::Cancel),
	};
	let datapack_name = ask_merged_datapack_name()?;

	let selection = &selection_items[selection];

	let (core_datapack, core_size) = get_core_datapack(&selection, &datapack_entries, |_| {})?;
	let (datapacks, sizes): (Vec<Datapack>, Vec<u64>) =
		get_other_datapack(&selection, &datapack_entries, |_| {});
	let total_size = core_size + sizes.iter().sum::<u64>();

	let temp_dir = tempfile::tempdir()?;
	let mut output_datapack = Datapack::from(temp_dir.path());

	for datapack in datapacks {
		if &datapack.name != selection {
			output_datapack = output_datapack.merge(datapack, |_| {})?;
		}
	}

	output_datapack = output_datapack.merge(core_datapack, |_| {})?;

	let output_path = get_output_path(&directory, &datapack_name);

	let compiling_bar = prepare_compiling_progress_bar(total_size);
	let options = prepare_zip_options();

	output_datapack.compile(&output_path, &options, |delta| compiling_bar.inc(delta))?;

	compiling_bar.finish();

	println!(
		"Compiled datapack to: '{}'",
		output_path.display().to_string().cyan()
	);

	Ok(())
}

fn ask_core_datapack(selection_items: &[String]) -> io::Result<Option<usize>> {
	Select::with_theme(&ColorfulTheme::default())
		.with_prompt("Please choose core datapack")
		.default(0)
		.items(&selection_items)
		.paged(true)
		.interact_opt()
}

fn ask_merged_datapack_name() -> io::Result<String> {
	Input::with_theme(&ColorfulTheme::default())
		.with_prompt("Merged datapack name")
		.default("merged_datapack".to_string())
		.allow_empty(false)
		.show_default(true)
		.interact()
}

fn prepare_compiling_progress_bar(size: u64) -> ProgressBar {
	let template = format!(
		"[{{elapsed}}] {} [{{wide_bar:.white}}] {{bytes}}/{{total_bytes}}",
		"Compiling".yellow().bold()
	);
	let style = ProgressStyle::default_bar().template(&template);
	ProgressBar::new(size).with_style(style)
}

use zip::write::FileOptions;
#[cfg(not(windows))]
fn prepare_zip_options() -> FileOptions {
	FileOptions::default()
		.compression_method(get_compression_method())
		.unix_permissions(0o775)
}

/// Window doesn't have concept of "unix permissions", if we try to create a file with unix permissions it will result in Inaccessible file permission.
#[cfg(windows)]
fn prepare_zip_options() -> FileOptions {
	FileOptions::default()
		.compression_method(get_compression_method())
}

fn get_selection_items(datapack_entries: DatapackIterator) -> (Vec<String>, Vec<DatapackLoader>) {
	datapack_entries
		.map(|entry| -> (String, DatapackLoader) {
			let name = os_str_to_string(&entry.file_name());
			let loader = DatapackLoader::new(entry.path()).unwrap();
			(name, loader)
		})
		.unzip()
}

fn get_core_datapack(
	name: &str,
	datapacks: &[DatapackLoader],
	event: impl Fn(u64) + Sync + Send + Copy,
) -> Result<(Datapack, u64), MergeError> {
	datapacks
		.par_iter()
		.find_any(|datapack| datapack.name == name)
		.cloned()
		.map(|loader| {
			let datapack = Datapack::generate(&loader.path, event).unwrap();
			loader.cleanup();
			datapack
		})
		.ok_or(MergeError::Other("Cannot find core datapack"))
}

fn get_other_datapack(
	name: &str,
	datapacks: &[DatapackLoader],
	event: impl Fn(u64) + Sync + Send + Copy,
) -> (Vec<Datapack>, Vec<u64>) {
	datapacks
		.par_iter()
		.filter(|loader| loader.name != name)
		.map(|loader| {
			let datapack = Datapack::generate(&loader.path, event).unwrap();
			loader.cleanup();
			datapack
		})
		.unzip()
}

fn get_output_path(directory: impl Into<PathBuf>, name: &str) -> PathBuf {
	let directory = directory.into();
	let output_file = PathBuf::from(format!("{}.zip", name));
	directory.join(output_file)
}
