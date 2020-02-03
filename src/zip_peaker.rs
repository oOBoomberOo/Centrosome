use indicatif::{ProgressBar};
use std::error::Error;
use std::fs::{create_dir_all, remove_dir_all, DirEntry, File};
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use zip::ZipArchive;

pub fn peak(
	entry: &DirEntry,
	temp_dir: &PathBuf,
	progress_bar: &ProgressBar,
) -> Result<(), Box<dyn Error>> {
	let node = entry.path();
	let name = node.file_name().unwrap();
	let location = temp_dir.join(name);

	create_dir_all(&location)?;

	let file = File::open(&node)?;
	let reader = BufReader::new(file);

	let mut archive = ZipArchive::new(reader)?;
	progress_bar.set_length(archive.len() as u64);

	for n in 0..archive.len() {
		let mut reader = archive.by_index(n)?;
		let zname = reader.sanitized_name();
		progress_bar.inc(1);

		let output = location.join(zname);

		if reader.name().ends_with('/') {
			create_dir_all(output)?;
		} else {
			if let Some(parent) = output.parent() {
				create_dir_all(parent)?;
			}

			let mut writer = File::create(output)?;
			io::copy(&mut reader, &mut writer)?;
		}
	}

	remove_dir_all(&location)?;

	progress_bar.finish_with_message("");

	Ok(())
}
