use std::path::PathBuf;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct Resource {
	location: PathBuf,
	data: String
}

impl Resource {
	pub fn new(location: PathBuf, datapack: &PathBuf) -> Resource {
		let data = read_to_string(&location).unwrap_or_default();
		let datapack_str = datapack.to_str().unwrap();
		let location_str = location.to_str().unwrap();
		let result_str = location_str.replace(datapack_str, "");
		let location = PathBuf::from(result_str);
		Resource { location, data }
	}

	pub fn from_data(data: &str, location: PathBuf) -> Resource {
		let data = data.to_owned();
		Resource { location, data }
	}
}