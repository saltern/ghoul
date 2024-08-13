use std::fs;
use std::fs::File;
use bmp_rust::bmp::{BMP, BITMAPFILEHEADER, DIBHEADER};

use crate::{
	PathBuf,
	SpriteData,
	bin_header,
	bin_header::BinHeader,
	sprite_compress,
};

const BITMAPCOREHEADER_SIZE: usize = 12;
const BMP_COLOR_24: usize = 3;
const BMP_COLOR_32: usize = 4;


pub fn get_png(source_file: &PathBuf) -> SpriteData {
	// Get info
	let file: File;
	match File::open(&source_file) {
		Ok(value) => file = value,
		_ => {
			println!("sprite_get::get_png() error: PNG file open error");
			println!("\tSkipped: {}", &source_file.display());
			return SpriteData::default();
		},
	}
	
	let mut decoder = png::Decoder::new(file);
	decoder.set_transformations(png::Transformations::STRIP_16);
	let mut reader = decoder.read_info().unwrap();
	
	let mut palette: Vec<u8> = vec![0; 256 * 3];
	
	// Get bytes
	let mut buffer = vec![0; reader.output_buffer_size()];
	let frame = reader.next_frame(&mut buffer).unwrap();
	
	let source_bytes = &buffer[..frame.buffer_size()];
	let mut pixel_vector: Vec<u8> = Vec::new();
	
	// Transfer color indices to pixel_vector
	match reader.info().color_type {
		png::ColorType::Grayscale => {
			pixel_vector = source_bytes.to_vec();
		},
		
		png::ColorType::Indexed => {
			pixel_vector = source_bytes.to_vec();
			match &reader.info().palette {
				Some(pal_data) => palette = pal_data.to_vec(),
				_ => (),
			}
		}
		
		png::ColorType::GrayscaleAlpha => {
			println!("Note: PNG has color type grayscale with alpha, will discard alpha");
			println!("\tFile: {}", &source_file.display());
			for pixel in 0..source_bytes.len() / 2 {
				pixel_vector.push(source_bytes[pixel * 2]);
			}
		},
		
		png::ColorType::Rgb => {
			println!("Note: PNG has color type RGB, will use red channel as grayscale");
			println!("\tFile: {}", &source_file.display());
			for pixel in 0..source_bytes.len() / 3 {
				pixel_vector.push(source_bytes[pixel * 3]);
			}
		},
		
		png::ColorType::Rgba => {
			println!("Note: PNG has color type RGBA, will use red channel as grayscale and discard alpha");
			println!("\tFile: {}", &source_file.display());
			for pixel in 0..source_bytes.len() / 4 {
				pixel_vector.push(source_bytes[pixel * 4]);
			}
		},
	}

	return SpriteData {
		width: reader.info().width as u16,
		height: reader.info().height as u16,
		bit_depth: 8,
		pixels: pixel_vector,
		palette: palette,
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
		println!("Warning: will not process RAW as its width was not specified");
		println!("\tSkipped: {}", &source_file.display());
		return SpriteData::default();
	}
	
	if height == 0 {
		println!("Warning: will not process RAW as its height was not specified");
		println!("\tSkipped: {}", &source_file.display());
		return SpriteData::default();
	}

	// All good, return raw data
	match fs::read(source_file) {
		Ok(data) => {
			return SpriteData {
				width: width,
				height: height,
				bit_depth: 8,
				pixels: data,
				palette: vec![],
			}
		},
		_ => {
			println!("sprite_get::get_raw() error: RAW file read error");
			println!("\tSkipped: {}", &source_file.display());
			return SpriteData::default();
		},
	}
}


pub fn get_bin(source_file: &PathBuf) -> SpriteData {
	// Figure out if the sprite is compressed or not
	let bin_data: Vec<u8>;
	match fs::read(source_file) {
		Ok(value) => bin_data = value,
		_ => {
			println!("sprite_get::get_bin() error: BIN file read error");
			println!("\tSkipped: {}", &source_file.display());
			return SpriteData::default();
		},
	}
	
	let header: BinHeader = bin_header::get_header(bin_data[0x0..0x10].to_vec());
	
	if header.compressed {
		return sprite_compress::decompress(bin_data, header);
	}
	
	else {
		let pointer: usize;
		
		// Embedded palette-- do those even work for uncompressed sprites?
		if header.clut == 0x20 {
			pointer = bin_header::HEADER_SIZE + (2u8.pow(header.bit_depth as u32) * 4) as usize;
		}
		else {
			pointer = bin_header::HEADER_SIZE;
		}
	
		let mut pixels: Vec<u8> = vec![0; bin_data.len() - pointer];
		
		pixels.copy_from_slice(&bin_data[pointer..]);
	
		return SpriteData {
			width: header.width,
			height: header.height,
			bit_depth: header.bit_depth,
			pixels: pixels,
			palette: vec![],
		}
	}
}


pub fn get_bmp(source_file: &PathBuf) -> SpriteData {
	// Not using BMP::new_from_file as it does not account for
	// failing to read from a file and will panic if it does
	
	// File read
	let bytes: Vec<u8>;
	match fs::read(source_file) {
		Ok(value) => bytes = value,
		_ => {
			println!("sprite_get::get_bmp() error: BMP file read error");
			println!("\tSkipped: {}", &source_file.display());
			return SpriteData::default();
		},
	}
	
	let mut bmp: BMP = BMP::new(50i32, 50u32, Some([0u8, 0u8, 0u8, 0u8]));
	bmp.contents = bytes;
	
	// Header reads
	let file_header: BITMAPFILEHEADER = BMP::get_header(&bmp);
	
	let dib_header: DIBHEADER;
	match BMP::get_dib_header(&bmp) {
		Ok(header) => dib_header = header,
		_ => {
			println!("sprite_get::get_bmp() error: Could not read DIB header");
			println!("\tSkipped: {}", &source_file.display());
			return SpriteData::default();
		},
	}
	
	// BPP check
	if dib_header.bitcount > 8 {
		println!("Warning: Skipping BMP as its color depth exceeds 8 bits per pixel");
		println!("\tSkipped: {}", &source_file.display());
		return SpriteData::default();
	}
	
	// Manual pixel read from bytes, accounts for padding
	let mut padding: usize = (dib_header.width % 4) as usize;
	if padding > 0 {
		padding = 4 - padding;
	}

	let mut pixel_vector: Vec<u8> = Vec::new();
	let width_bytes: usize = (dib_header.width + padding as u32) as usize;
	
	for row in (0..dib_header.height as usize).rev() {
		for pixel in 0..dib_header.width as usize {
			let row_start: usize = width_bytes * row;
			pixel_vector.push(bmp.contents[file_header.bfOffBits as usize + row_start + pixel]);
		}
	}
	
	// Invalid BMP	
	if std::cmp::max(dib_header.width, dib_header.height.abs() as u32) > u16::MAX as u32 {
		println!("sprite_get::get_bmp() error: image dimensions exceed sprite maximum of 65535px per side");
		println!("\tSkipped: {}", &source_file.display());
		return SpriteData::default();
	}
	
	if pixel_vector.len() != (dib_header.width * dib_header.height.abs() as u32) as usize {
		println!("sprite_get::get_bmp() error: bad BMP: pixel count mismatches image dimensions, result may differ");
		println!("\tFile: {}", &source_file.display());
		pixel_vector.resize((dib_header.width * dib_header.height.abs() as u32) as usize, 0u8);
	}
	
	// Palette read
	let flags_offset: usize;
	match dib_header.compression {
		Some(value) => match &value as &str {
			"BI_BITFIELDS" => flags_offset = 12,
			"BI_ALPHABITFIELDS" => flags_offset = 16,
			_ => flags_offset = 0,
		},
		
		None => flags_offset = 0,
	}
	
	// Bytes per palette color
	let index: usize = 14 + dib_header.size as usize + flags_offset;
	let color_size: usize;
	if dib_header.size as usize == BITMAPCOREHEADER_SIZE {
		color_size = BMP_COLOR_24;
	}
	else {
		color_size = BMP_COLOR_32;
	}
	
	// How many colors to read from BMP color table
	let color_count: usize;
	match dib_header.ClrUsed {
		Some(value) => match value {
			0 => color_count = 2u16.pow(dib_header.bitcount as u32) as usize,
			_ => color_count = value as usize,
		},
		
		None => color_count = 2u16.pow(dib_header.bitcount as u32) as usize,
	}
	
	// Create and populate palette
	let mut palette: Vec<u8> = vec![0; 256 * 3];
	
	for color in 0..color_count {
		palette[3 * color + 0] = bmp.contents[index + (color_size * color + 2)];
		palette[3 * color + 1] = bmp.contents[index + (color_size * color + 1)];
		palette[3 * color + 2] = bmp.contents[index + (color_size * color + 0)];
	}
	
	return SpriteData {
		width: dib_header.width as u16,
		height: dib_header.height as u16,
		bit_depth: 8,
		pixels: pixel_vector,
		palette: palette,
	}
}