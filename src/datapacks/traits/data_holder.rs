use super::FileType;

pub trait DataHolder {
	fn data(&self) -> &Option<Vec<u8>>;

	fn file_type(&self) -> FileType {
		if self.data().is_none() {
			FileType::Folder
		} else {
			FileType::File
		}
	}

	fn get_data(&self) -> String {
		match self.data().to_owned() {
			Some(v) => String::from_utf8(v).unwrap_or_default(),
			None => String::default(),
		}
	}
}
