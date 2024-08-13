use crate::PathBuf;

#[derive(Clone)]
pub struct Parameters {
	pub directory_mode: bool,
	pub source_path: PathBuf,
	pub target_path: PathBuf,
	pub palette_file: PathBuf,
	pub source_format: SpriteFormat,
	pub target_format: SpriteFormat,
	pub palette_transfer: bool,
	pub uncompressed: bool,
	pub reindex: bool,
	pub verbose: bool,
	pub overwrite: bool,
}

#[derive(PartialEq, Copy, Clone)]
pub enum SpriteFormat {
	NONE,
	PNG,
	RAW,
	BIN,
	BMP,
}

#[derive(Debug)]
pub struct SpriteData {
	pub width: u16,
	pub height: u16,
	pub bit_depth: u16,
	pub pixels: Vec<u8>,
	pub palette: Vec<u8>,
}

impl Default for SpriteData {
	fn default() -> SpriteData {
		SpriteData {
			width: 0,
			height: 0,
			bit_depth: 8,
			pixels: Vec::new(),
			palette: Vec::new(),
		}
	}
}

pub struct CompressedData {
	pub iterations: usize,
	pub stream: Vec<u8>,
}