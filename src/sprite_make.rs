use std::io::{Write, BufWriter};
use std::fs;
use std::fs::File;

use crate::{
	PathBuf,
	Parameters,
	SpriteData,
	SpriteFormat,
	shared_types::CompressedData,
	sprite_compress,
};


fn overwrite_blocked(target_path: &PathBuf, overwrite: bool) -> bool {
	if !overwrite {
		match target_path.try_exists() {
			Ok(true) => {
				println!("'{}' already exists. Use -w to allow overwriting files.", target_path.to_str().unwrap());
				return true;
			},
			
			Ok(false) => (),
			
			_ => {
				println!("sprite_make::overwrite_blocked() error: Errored while checking if file already exists");
				println!("\tSkipped: {}", target_path.display());
				return true;
			},
		}
	}
	
	return false;
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


fn bmp_header(width: u16, height: u16) -> Vec<u8> {
	let mut bmp_data: Vec<u8> = Vec::new();
	
	// BITMAPFILEHEADER
	// 2 bytes, "BM"
	bmp_data.push(0x42);
	bmp_data.push(0x4d);
	
	// 4 bytes, size of the bitmap in bytes
	// 794 is:
	// 14 bytes - BITMAPFILEHEADER
	// 12 bytes - DIBHEADER of type BITMAPCOREHEADER
	// 768 bytes - Color Table of 256 colors of 3 bytes each
	let bmp_file_size: [u8; 4] = (794 + (width + width % 4) as u32 * height as u32).to_le_bytes();
	for byte in 0..4 {
		bmp_data.push(bmp_file_size[byte]);
	}
	
	// 2 bytes each for bfReserved1 and 2
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	
	// 4 bytes, offset to pixel array
	bmp_data.push(0x1A);
	bmp_data.push(0x03);
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	
	// DIBHEADER (BITMAPCOREHEADER)
	// 4 bytes, 12
	bmp_data.push(0x0C);
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	
	// 2 bytes, image width
	bmp_data.push(width as u8);
	bmp_data.push((width >> 8) as u8);
	
	// 2 bytes, image height
	bmp_data.push(height as u8);
	bmp_data.push((height >> 8) as u8);
	
	// 2 bytes, planes
	bmp_data.push(0x01);
	bmp_data.push(0x00);
	
	// 2 bytes, bpp
	bmp_data.push(0x08);
	bmp_data.push(0x00);
	
	return bmp_data;
}


pub fn make_png(parameters: Parameters, data: SpriteData) {
	// Set target filename
	let mut target_path: PathBuf = parameters.target_path;
	target_path.push(parameters.source_path.file_stem().unwrap());
	target_path.set_extension("png");
	
	if overwrite_blocked(&target_path, parameters.overwrite) {
		return
	}
	
	// Make PNG
	let png_file: File;
	match File::create(&target_path) {
		Ok(file) => png_file = file,
		
		_ => {
			println!("sprite_make::make_png() error: Could not create target PNG file");
			println!("\tSkipped: {}", &target_path.display());
			return;
		},
	}

	let ref mut buffer = BufWriter::new(png_file);
	
	let mut encoder = png::Encoder::new(buffer, data.width as u32, data.height as u32);
	encoder.set_depth(png::BitDepth::Eight);
	encoder.set_color(png::ColorType::Grayscale);
	
	// Palette transfer
	if parameters.palette_transfer && data.palette.len() == 768 {
		encoder.set_color(png::ColorType::Indexed);
		encoder.set_palette(data.palette);
	}
	
	else if !data.palette.is_empty() {
		encoder.set_color(png::ColorType::Indexed);
		encoder.set_palette(data.palette);
	}
	
	else if !parameters.palette_file.as_os_str().is_empty() {
		match fs::read(parameters.palette_file) {
			Ok(data) => {
				encoder.set_color(png::ColorType::Indexed);
			
				// Load palette
				let mut act_data: Vec<u8> = data;
				
				// Fill in with black if it doesn't contain enough colors
				act_data.resize(768, 0u8);
				
				encoder.set_palette(act_data);
			},
				
			_ => {
				println!("sprite_make::make_png() error: Could not read source palette file, ignoring");
			},
		}
	}
	
	let mut writer = encoder.write_header().expect("sprite_make::make_png() error: Could not write PNG header");
	let mut write_data: Vec<u8> = data.pixels.clone();
	write_data.resize((data.width as u32 * data.height as u32) as usize, 0u8);
	writer.write_image_data(&data.pixels).unwrap();
}


pub fn make_raw(parameters: Parameters, data: SpriteData) {
	// Set target filename, do not append -W-X-H-Y if coming from raw
	let mut target_path: PathBuf = parameters.target_path;
	if parameters.source_format == SpriteFormat::RAW {
		target_path.push(parameters.source_path);
	}
	
	else {
		let file_stem: &str = parameters.source_path.file_stem().unwrap().to_str().unwrap();
		target_path.push(format!("{}-W-{}-H-{}.raw", file_stem, data.width, data.height));
	}
	
	if overwrite_blocked(&target_path, parameters.overwrite) {
		return;
	}
	
	// Make RAW
	let raw_file: File;
	
	match File::create(&target_path) {
		Ok(file) => raw_file = file,
		_ => {
			println!("sprite_make::make_raw() error: Could not create target RAW file");
			println!("\tSkipped: {}", &target_path.display());
			return;
		},
	}
	
	let ref mut buffer = BufWriter::new(raw_file);
	
	for pixel in 0..data.pixels.len() {
		let _ = buffer.write(&[data.pixels[pixel]]);
	}
		
	let _ = buffer.flush();
}


pub fn make_bin(parameters: Parameters, data: SpriteData) {
	// Set target filename
	let mut target_path = parameters.target_path;
	target_path.push(parameters.source_path.file_stem().unwrap());
	target_path.set_extension("bin");
	
	if overwrite_blocked(&target_path, parameters.overwrite) {
		return;
	}
	
	// Make BIN
	let bin_file: File;
	
	match File::create(&target_path) {
		Ok(file) => bin_file = file,
		_ => {
			println!("sprite_make::make_bin() error: Could not create target BIN file");
			println!("\tSkipped: {}", &target_path.display());
			return;
		},
	}
	
	let ref mut buffer = BufWriter::new(bin_file);
	
	// Header
	let header_vector: Vec<u8> = bin_header(data.width, data.height, parameters.uncompressed);
	
	for item in 0..header_vector.len() {
		let _ = buffer.write(&[header_vector[item]]);
	}
	
	// Uncompressed mode
	if parameters.uncompressed {
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


pub fn make_bmp(parameters: Parameters, data: SpriteData) {
	// Set target filename
	let mut target_path: PathBuf = parameters.target_path;
	target_path.push(parameters.source_path.file_stem().unwrap());
	target_path.set_extension("bmp");
	
	if overwrite_blocked(&target_path, parameters.overwrite) {
		return;
	}

	// BITMAPFILEHEADER, BITMAPCOREHEADER
	let header: Vec<u8> = bmp_header(data.width, data.height);
	
	// Color table
	let mut color_table: Vec<u8> = Vec::with_capacity(768);
	
	// Palette transfer
	if parameters.palette_transfer && data.palette.len() == 768 {
		for color in 0..256 {
			color_table.push(data.palette[3 * color + 2]);
			color_table.push(data.palette[3 * color + 1]);
			color_table.push(data.palette[3 * color + 0]);
		}
	}
	
	// Grayscale
	else if parameters.palette_file.as_os_str().is_empty() {
		for color in 0..256 {
			color_table.push(color as u8);
			color_table.push(color as u8);
			color_table.push(color as u8);
		}
	}
	
	else {
		// Load palette
		match fs::read(parameters.palette_file) {
			Ok(data) => {
				let mut act_data: Vec<u8> = data;
		
				// Fill in with black if palette doesn't contain enough colors
				act_data.resize(768, 0u8);
	
				// BGR
				for color in 0..256 {
					color_table.push(act_data[3 * color + 2]);
					color_table.push(act_data[3 * color + 1]);
					color_table.push(act_data[3 * color + 0]);
				}
			},
				
			_ => {
				println!("sprite_make::make_bmp() error: Could not read source palette file, ignoring");
			},
		}
	}
	
	// Write out
	let bmp_file: File;
	match File::create(&target_path) {
		Ok(file) => bmp_file = file,
		_ => {
			println!("sprite_make::make_bmp() error: Could not create target file");
			println!("\tSkipped: {}", &target_path.display());
			return;
		},
	}
		
	let mut buffer = BufWriter::new(bmp_file);
	
	let _ = buffer.write_all(&header);
	let _ = buffer.write_all(&color_table);
	
	let mut padding: usize = (data.width % 4) as usize;
	if padding > 0 {
		padding = 4 - padding;
	}
	
	let width: usize = data.width as usize;
	// Upside-down write with padding
	for y in (0..data.height as usize).rev() {
		let row_start: usize = y * width;		
		let _ = buffer.write_all(&data.pixels[row_start..row_start + width]);
		let _ = buffer.write_all(&vec![0u8; padding]);
	}
	
	match buffer.flush() {
		Ok(_) => (),
		_ => {
			println!("sprite_make::make_bmp() error: Could not write to target file");
			println!("\tFile: {}", &target_path.display());
		}
	}
}