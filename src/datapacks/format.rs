#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EncodingFormat {
	Utf8,
	Utf16
}

impl Default for EncodingFormat {
	fn default() -> EncodingFormat {
		EncodingFormat::Utf8
	}
}