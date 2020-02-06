use crate::zipper::Zipper;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{DirEntry};
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/**
 * Progress bar template for extracting zip file progress
 * Use in combination with Multi-Progress
 */
fn create_progress_bar(
	multi_progress: &MultiProgress,
	entry: &DirEntry,
	width: usize,
) -> Result<ProgressBar> {
	let entry_name = get_path_name(&entry.path()).bright_green().bold();
	let template = format!("[{{elapsed}}] {:<width$} [{{wide_bar:.cyan/white}}] {{bytes:>8}}/{{total_bytes:8}} {{percent:>3}}%", entry_name, width = width);
	let style = ProgressStyle::default_bar()
		.template(&template)
		.progress_chars("#>-");
	let progress_bar = ProgressBar::new(1000).with_style(style);
	Ok(multi_progress.add(progress_bar))
}

/**
 * Create Zipper and Progress bar for that zipper
 */
pub fn create_zipper(
	entry: &DirEntry,
	length: usize,
	multi_progress: &MultiProgress,
) -> Result<Zipper> {
	let progress_bar = create_progress_bar(&multi_progress, &entry, length)?;
	let zipper = Zipper::new(entry.path()).set_progress_bar(progress_bar);

	Ok(zipper)
}

/**
 * Read directory and filter in a valid datapack
 */
pub fn read_directory(directory: &Path, progress_bar: ProgressBar) -> Result<Vec<DirEntry>> {
	let ok_directory: Vec<DirEntry> = directory
		.read_dir()?
		.filter_map(|entry| entry.ok())
		.collect();

	progress_bar.set_length(ok_directory.len() as u64);

	let result: Vec<DirEntry> = ok_directory
		.into_par_iter()
		.filter(|entry| {
			let result = file_metadata(&entry);
			progress_bar.inc(1);
			result
		})
		.collect();

	progress_bar.finish_with_message("Finished loading metadata!");

	Ok(result)
}

pub fn get_longest_name_length(list: &[String]) -> usize {
	let mut result: Vec<usize> = list.par_iter().map(|item| item.len()).collect();
	result.sort();
	match result.last() {
		Some(&x) => x,
		None => 0,
	}
}

/**
 * Check if `entry` is a valid datapack
 */
fn file_metadata(entry: &DirEntry) -> bool {
	let path = entry.path();

	if path.is_dir() {
		let data_folder = path.join("data");
		let mcmeta_file = path.join("pack.mcmeta");

		mcmeta_file.exists()
			&& mcmeta_file.is_file()
			&& data_folder.exists()
			&& data_folder.is_dir()
	} else if path.is_file() {
		let extension = path.extension();

		if extension == Some(OsStr::new("zip")) {
			let zipper = Zipper::new(path);
			// ! `zip-rs` use / at the end of path to indicate directory
			let (_, data_folder) = zipper.peak("data/");
			let (mcmeta_file, _) = zipper.peak("pack.mcmeta");

			data_folder && mcmeta_file
		} else {
			false
		}
	} else {
		false
	}
}

/**
 * Get file/directory name without its parent
 */
pub fn get_path_name(path: &PathBuf) -> String {
	path.file_name()
		.unwrap_or_default()
		.to_str()
		.unwrap_or_default()
		.to_string()
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;
	use super::{get_path_name, get_longest_name_length};

	#[test]
	fn test_dio_file_path() {
		let value = PathBuf::from("/oh/you/are/approaching/me.question");
		assert_eq!(get_path_name(&value), String::from("me.question"))
	}

	#[test]
	fn test_jotaro_dir_path() {
		let value = PathBuf::from("/i/cant/beat/the/shit/out/of/you/with/out/getting/closer");
		assert_eq!(get_path_name(&value), String::from("closer"))
	}

	#[test]
	fn get_longest_jojo_name() {
		let jojos: Vec<String> = vec![
			"Jonathan Joestar",
			"Joseph Joestar",
			"Jotaro Kujo",
			"Josuke Higashikata",
			"Giorno Giovanna",
			"Jolyne Cujoh",
			"Johnny Joestar",
			"Josuke Higashikata"
		].iter().map(|jojo| String::from(*jojo)).collect();
		assert_eq!(get_longest_name_length(jojos.as_slice()), 18);
	}
}