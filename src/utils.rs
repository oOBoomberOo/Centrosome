use crate::zipper::Zipper;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs::{DirEntry};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use rayon::prelude::*;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn create_progress_bar(multi_progress: &MultiProgress, entry: &DirEntry, width: usize) -> Result<ProgressBar> {
	let entry_name = entry_name(entry)?.bright_green().bold();
	let template = format!("[{{elapsed}}] {:<width$} [{{wide_bar:.cyan/white}}] {{pos:>4}}/{{len:4}} {{percent:>3}}% ({{eta:^4}})", entry_name, width = width);
	let style = ProgressStyle::default_bar()
		.template(&template)
		.progress_chars("#>-");
	let progress_bar = ProgressBar::new(1000).with_style(style);
	Ok(multi_progress.add(progress_bar))
}

pub fn create_zipper(entry: &DirEntry, temp_directory: &Path, length: usize, multi_progress: &MultiProgress) -> Result<Zipper> {
	let progress_bar = create_progress_bar(&multi_progress, &entry, length)?;
	let _directory = temp_directory.join(entry_name(entry)?);
	let zipper = Zipper::new(entry.path()).set_progress_bar(progress_bar);

	Ok(zipper)
}

pub fn read_directory(directory: &Path) -> Result<Vec<DirEntry>> {
	Ok(directory
		.read_dir()?
		.filter_map(|entry| entry.ok())
		.filter(file_metadata)
		.collect())
}

pub fn entry_name(entry: &DirEntry) -> Result<String> {
	let result = entry.file_name().to_str().ok_or("Can't read file name")?.to_owned();
	Ok(result)
}

pub fn get_longest_name_length(list: &[String]) -> usize {
	let mut result: Vec<usize> = list.par_iter().map(|item| item.len()).collect();
	result.sort();
	match result.last() {
		Some(&x) => x,
		None => 0
	}
}

fn file_metadata(entry: &DirEntry) -> bool {
	let path = entry.path();
	if path.is_dir() {
		let data_folder = path.join("data");
		let mcmeta_file = path.join("pack.mcmeta");

		mcmeta_file.exists() && mcmeta_file.is_file() && data_folder.exists() && data_folder.is_dir()
	}
	else if path.is_file() {
		if path.extension() == Some(OsStr::new("zip")) {
			let zipper = Zipper::new(path);
			// ! `zip-rs` use / at the end of path to indicate directory
			let (_, data_folder) = zipper.peak("data/");
			let (mcmeta_file, _) = zipper.peak("pack.mcmeta");

			data_folder && mcmeta_file
		}
		else {
			false
		}
	}
	else {
		false
	}
}

pub fn get_path_name(path: &PathBuf) -> String {
	path.file_name()
		.unwrap_or_default()
		.to_str()
		.unwrap_or_default()
		.to_string()
}