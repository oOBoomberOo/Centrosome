use crate::zipper::{Zipper, ZipperError};
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{create_dir_all, remove_dir_all, DirEntry};
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn create_progress_bar(
	multi_progress: &MultiProgress,
	entry: &DirEntry,
	width: usize,
) -> Result<ProgressBar> {
	let entry_name = entry_name(entry)?.bright_green().bold();
	let template = format!("[{{elapsed}}] {:<width$} [{{wide_bar:.cyan/white}}] {{bytes:>8}}/{{total_bytes:8}} {{percent:>3}}%", entry_name, width = width);
	let style = ProgressStyle::default_bar()
		.template(&template)
		.progress_chars("#>-");
	let progress_bar = ProgressBar::new(1000).with_style(style);
	Ok(multi_progress.add(progress_bar))
}

pub fn create_zipper(
	entry: &DirEntry,
	length: usize,
	multi_progress: &MultiProgress,
) -> Result<Zipper> {
	let format = match entry.path().extension() {
		None => CompressionFormat::Directory,
		Some(os_str) => match os_str.to_str().expect("Invalid file content") {
			"zip" => CompressionFormat::Zip,
			"gz" => CompressionFormat::Tar,
			_ => CompressionFormat::Unknown(entry.path()),
		},
	};

	if let CompressionFormat::Unknown(path) = format {
		return Err(Box::new(ZipperError::UnknownFormat(path)));
	}

	let progress_bar = create_progress_bar(&multi_progress, &entry, length)?;
	let zipper = Zipper::new(entry.path(), format).set_progress_bar(progress_bar);

	Ok(zipper)
}

pub fn read_directory(directory: &Path, progress_bar: ProgressBar) -> Result<Vec<DirEntry>> {
	let temp_dir = std::env::temp_dir().join("datapack_merger-read-directory");
	create_dir_all(&temp_dir)?;

	let ok_directory: Vec<DirEntry> = directory
		.read_dir()?
		.filter_map(|entry| entry.ok())
		.collect();

	progress_bar.set_length(ok_directory.len() as u64);

	let result: Vec<DirEntry> = ok_directory
		.into_par_iter()
		.filter(|entry| file_metadata(&entry, &temp_dir, &progress_bar))
		.collect();

	remove_dir_all(&temp_dir)?;
	progress_bar.finish_with_message("Finished loading metadata!");

	Ok(result)
}

pub fn entry_name(entry: &DirEntry) -> Result<String> {
	let result = entry
		.file_name()
		.to_str()
		.ok_or("Can't read file name")?
		.to_owned();
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

use crate::zipper::CompressionFormat;

fn file_metadata(entry: &DirEntry, temp_dir: &PathBuf, progress_bar: &ProgressBar) -> bool {
	let path = entry.path();
	let result = {
		if path.is_dir() {
			let data_folder = path.join("data");
			let mcmeta_file = path.join("pack.mcmeta");

			mcmeta_file.exists()
				&& mcmeta_file.is_file()
				&& data_folder.exists()
				&& data_folder.is_dir()
		} else if path.is_file() {
			let name = get_path_name(&path);
			let temp_dir = temp_dir.join(name);
			let extension = path.extension();

			if extension == Some(OsStr::new("zip")) {
				let zipper = Zipper::new(path, CompressionFormat::Zip);
				// ! `zip-rs` use / at the end of path to indicate directory
				let (_, data_folder) = zipper.peak("data/", &temp_dir);
				let (mcmeta_file, _) = zipper.peak("pack.mcmeta", &temp_dir);

				data_folder && mcmeta_file
			} else if extension == Some(OsStr::new("gz")) {
				let zipper = Zipper::new(path, CompressionFormat::Tar);
				// ! `zip-rs` use / at the end of path to indicate directory
				let (_, data_folder) = zipper.peak("data/", &temp_dir);
				let (mcmeta_file, _) = zipper.peak("pack.mcmeta", &temp_dir);

				data_folder && mcmeta_file
			} else {
				false
			}
		} else {
			false
		}
	};

	progress_bar.inc(1);
	result
}

pub fn get_path_name(path: &PathBuf) -> String {
	path.file_name()
		.unwrap_or_default()
		.to_str()
		.unwrap_or_default()
		.to_string()
}
