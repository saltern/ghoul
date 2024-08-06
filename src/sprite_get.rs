use std::fs;
use std::fs::File;

use crate::{
	PathBuf,
	SpriteData,
	sprite_compress,
};

pub fn get_png(source_file: &PathBuf) -> SpriteData {
	// Get info	
	let mut decoder = png::Decoder::new(File::open(&source_file).expect("sprite_get::get_png(): PNG file open error"));
	decoder.set_transformations(png::Transformations::STRIP_16);
	
	let mut reader = decoder.read_info().unwrap();

	// Get bytes
	let mut buffer = vec![0; reader.output_buffer_size()];
	let frame = reader.next_frame(&mut buffer).unwrap();
	
	let source_bytes = &buffer[..frame.buffer_size()];
	let mut index_vector: Vec<u8> = Vec::new();
	
	// Transfer indices to index_vector
	match reader.info().color_type {
		png::ColorType::Grayscale | png::ColorType::Indexed => {
			index_vector = source_bytes.to_vec();
		},
		
		png::ColorType::GrayscaleAlpha => {
			println!("Note: '{}' has color type grayscale with alpha, will discard alpha", &source_file.display());
			for pixel in 0..source_bytes.len() / 2 {
				index_vector.push(source_bytes[pixel * 2]);
			}
		},
		
		png::ColorType::Rgb => {
			println!("Note: '{}' has color type RGB- will use red channel as grayscale", &source_file.display());
			for pixel in 0..source_bytes.len() / 3 {
				index_vector.push(source_bytes[pixel * 3]);
			}
		},
		
		png::ColorType::Rgba => {
			println!("Note: '{}' has color type RGBA- will use red channel as grayscale and discard alpha", &source_file.display());
			for pixel in 0..source_bytes.len() / 4 {
				index_vector.push(source_bytes[pixel * 4]);
			}
		},
	}

	return SpriteData {
		width: reader.info().width as u16,
		height: reader.info().height as u16,
		pixels: index_vector,
	}
}


pub fn get_raw(source_file: &PathBuf) -> SpriteData {
	// Find if the RAW file has specified its dimensions
	let mut width: u16 = 0;
	let mut height: u16 = 0;
	
	let file_name: String = source_file.file_stem().unwrap().to_str().unwrap().to_lowercase();	
	let file_name_pieces: Vec<&str> = file_name.split("-").collect();
	let piece_count: usize = file_name_pieces.len();
	
	for piece in 0..piece_count {
		// Width
		if file_name_pieces[piece] == "w" && piece + 1 < piece_count {
			width = file_name_pieces[piece + 1].parse::<u16>().unwrap_or(0);
		}
		
		// Height
		if file_name_pieces[piece] == "h" && piece + 1 < piece_count {
			height = file_name_pieces[piece + 1].parse::<u16>().unwrap_or(0);
		}
	}
	
	if width == 0 {
		println!("Warning: will not process '{}' as its width was not specified.", &source_file.display());
		return SpriteData::default();
	}
	
	if height == 0 {
		println!("Warning: will not process '{}' as its height was not specified.", &source_file.display());
		return SpriteData::default();
	}

	// All good, return raw data
	let data: Vec<u8> = fs::read(source_file).expect("sprite_get::get_raw() error: RAW file read error");
	
	return SpriteData {
		width: width,
		height: height,
		pixels: data,
	}
}


pub fn get_bin(source_file: &PathBuf) -> SpriteData {
	// Figure out if the sprite is compressed or not
	let bin_data: Vec<u8> = fs::read(source_file).expect("sprite_get::get_bin() error: BIN file read error");
	
	if bin_data[0] == 1 {
		return sprite_compress::decompress(bin_data);
	}
	
	else {
		// Read dimensions from header
		let dimensions: (u16, u16) = (
			u16::from_le_bytes([bin_data[6], bin_data[7]]),
			u16::from_le_bytes([bin_data[8], bin_data[9]]));
		
		let mut pixels: Vec<u8> = vec![0; bin_data.len() - 16];
		
		pixels.copy_from_slice(&bin_data[16..]);
	
		return SpriteData {
			width: dimensions.0,
			height: dimensions.1,
			pixels: pixels,
		}
	}
}