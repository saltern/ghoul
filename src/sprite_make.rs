use std::io::{Write, BufWriter};
use std::fs;
use std::fs::File;

use crate::{
	PathBuf,
	SpriteData,
	shared_types::CompressedData,
	sprite_compress,
};


pub fn make_png(source_file: PathBuf, mut target_path: PathBuf, data: SpriteData, palette: PathBuf, overwrite: bool) {
	// Set target filename	
	target_path.push(source_file.file_stem().unwrap());
	target_path.set_extension("png");
	
	// Overwrite check
	if !overwrite {
		match target_path.try_exists() {
			Ok(true) => {
				println!("File already exists: '{}'. Use -w to allow overwriting files.", target_path.to_str().unwrap());
				return;
			},
			
			Ok(false) => (),
			
			_ => {
				println!("sprite_make::make_png() error: Errored while checking if file already exists. Skipping file.");
				return;
			},
		}
	}
	
	// Make PNG
	let png_file = File::create(target_path).expect("main::make_sprite_png() error: Could not create target PNG file");
	let ref mut buffer = BufWriter::new(png_file);
	
	let mut encoder = png::Encoder::new(buffer, data.width as u32, data.height as u32);
	encoder.set_depth(png::BitDepth::Eight);
	
	if palette.as_os_str().is_empty() {
		encoder.set_color(png::ColorType::Grayscale);
	}
	
	else {
		encoder.set_color(png::ColorType::Indexed);
		
		// Load palette
		let mut act_data: Vec<u8> = fs::read(palette).expect("sprite_make::make_png() error: Could not read source palette file");
		
		// Fill in with black if palette doesn't contain enough colors
		act_data.resize(768, 0u8);
		
		// Write palette to encoder
		encoder.set_palette(act_data);
	}
	
	let mut writer = encoder.write_header().expect("main::make_sprite_png() error: Could not write PNG header");
	writer.write_image_data(&data.pixels).unwrap();
}


pub fn make_raw(source_file: PathBuf, mut target_path: PathBuf, data: SpriteData, overwrite: bool, from_raw: bool) {
	// Set target filename, do not append -W-X-H-Y if coming from raw
	if from_raw {
		target_path.push(source_file);
	}
	
	else {
		let file_stem: &str = source_file.file_stem().unwrap().to_str().unwrap();
		target_path.push(format!("{}-W-{}-H-{}.raw", file_stem, data.width, data.height));
	}
	
	// Overwrite check
	if !overwrite {
		match target_path.try_exists() {
			Ok(true) => {
				println!("File already exists: '{}'. Use -w to allow overwriting files.", target_path.to_str().unwrap());
				return;
			},
			
			Ok(false) => (),
			
			_ => {
				println!("sprite_make::make_png() error: Errored while checking if file already exists. Skipping file.");
				return;
			},
		}
	}
	
	// Make RAW
	let raw_file = File::create(target_path).expect("main::make_sprite_raw() error: Could not create target RAW file");
	let ref mut buffer = BufWriter::new(raw_file);
	
	for pixel in 0..data.pixels.len() {
		let _ = buffer.write(&[data.pixels[pixel]]);
	}
		
	let _ = buffer.flush();
}


pub fn make_bin(source_file: PathBuf, mut target_path: PathBuf, data: SpriteData, uncompressed: bool, overwrite: bool) {
	// Set target filename
	target_path.push(source_file.file_stem().unwrap());
	target_path.set_extension("bin");
	
	// Overwrite check
	if !overwrite {
		match target_path.try_exists() {
			Ok(true) => {
				println!("File already exists: '{}'. Use -w to allow overwriting files.", target_path.to_str().unwrap());
				return;
			},
			
			Ok(false) => (),
			
			_ => {
				println!("sprite_make::make_png() error: Errored while checking if file already exists. Skipping file.");
				return;
			},
		}
	}
	
	// Make BIN
	let bin_file = File::create(target_path).expect("main::make_sprite_bin() error: Could not create target BIN file");
	let ref mut buffer = BufWriter::new(bin_file);
	
	// Header
	let header_vector: Vec<u8> = bin_header(data.width, data.height, uncompressed);
	
	for item in 0..header_vector.len() {
		let _ = buffer.write(&[header_vector[item]]);
	}
	
	// Uncompressed mode
	if uncompressed {
		let _ = buffer.write_all(&data.pixels);
	}
	
	// Compressed mode
	else {
		let compressed_data: CompressedData = sprite_compress::compress(data);
		
		// Yes, this is a u32 split across two LE u16s.
		let iterations_u32: u32 = compressed_data.iterations as u32;
		let _ = buffer.write_all(&[
			(iterations_u32 >> 16) as u8, // BB
			(iterations_u32 >> 24) as u8, // AA
			iterations_u32 as u8, // DD
			(iterations_u32 >> 8) as u8, // CC
		]);
		
		// LE write
		let mut byte: usize = 0;
		let length: usize = compressed_data.stream.len();
		
		while byte + 1 < length {
			let _ = buffer.write_all(&[
				compressed_data.stream[byte + 1],
				compressed_data.stream[byte]
			]);
			
			byte += 2;
		}
		
		if byte < length {
			compressed_data.stream[byte];
		}
	}
}


fn bin_header(width: u16, height: u16, uncompressed: bool) -> Vec<u8> {
	let mut header_vector: Vec<u8> = Vec::new();

	// mode
	if uncompressed {
		header_vector.push(0u8);
		header_vector.push(0u8);
	}
	
	else {
		header_vector.push(1u8);
		header_vector.push(0u8);
	}
	
	// clut, pix
	header_vector.push(0u8);
	header_vector.push(0u8);
	header_vector.push(8u8);
	header_vector.push(0u8);
	
	// width
	header_vector.push(width as u8);
	header_vector.push((width >> 8) as u8);
	
	// height
	header_vector.push(height as u8);
	header_vector.push((height >> 8) as u8);
	
	// tw, th, hash
	header_vector.push(0u8);
	header_vector.push(8u8);
	header_vector.push(0u8);
	header_vector.push(8u8);
	header_vector.push(0u8);
	header_vector.push(0u8);
	
	return header_vector;
}