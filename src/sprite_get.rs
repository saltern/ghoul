use std::fs;
use std::fs::File;
use bmp_rust::bmp::{BMP, BITMAPFILEHEADER, DIBHEADER};

use crate::{
	PathBuf,
	SpriteData,
	bin_header,
	bin_header::BinHeader,
	sprite_compress,
	sprite_transform,
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
	
	let mut palette: Vec<u8> = Vec::new();
	
	// Get bytes
	let mut buffer = vec![0; reader.output_buffer_size()];
	let frame = reader.next_frame(&mut buffer).unwrap();
	
	let source_bytes: Vec<u8> = buffer[..frame.buffer_size()].to_vec();
	let mut pixel_vector: Vec<u8> = Vec::new();
	
	// Transfer color indices to pixel_vector
	match reader.info().color_type {
		png::ColorType::Grayscale => {
			pixel_vector = source_bytes;
			
		},
		
		png::ColorType::Indexed => {
			pixel_vector = source_bytes;
			match &reader.info().palette {
				Some(pal_data) => {
					let temp_pal: Vec<u8> = pal_data.to_vec();
					let color_count: usize = temp_pal.len() / 3;
					let mut alpha_vec: Vec<u8> = Vec::new();
					
					match &reader.info().trns {
						Some(alpha) => {
							alpha_vec = alpha.to_vec();
						},
						
						_ => (),
					}
					
					alpha_vec.resize(color_count, 0x80);
					palette = vec![0; color_count * 4];
					
					for index in 0..color_count {
						palette[4 * index + 0] = temp_pal[3 * index + 0];
						palette[4 * index + 1] = temp_pal[3 * index + 1];
						palette[4 * index + 2] = temp_pal[3 * index + 2];
						palette[4 * index + 3] = alpha_vec[index];
					}
				},
					
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

	// Bit depth management
	let mut bit_depth: u16 = 8;
	match reader.info().bit_depth {
		png::BitDepth::One => {
			bit_depth = 1;
			pixel_vector = sprite_transform::bpp_from_1(pixel_vector, false);
		},
		
		png::BitDepth::Two => {
			bit_depth = 2;
			pixel_vector = sprite_transform::bpp_from_2(pixel_vector, false);
		},
		
		png::BitDepth::Four => {
			bit_depth = 4;
			pixel_vector = sprite_transform::bpp_from_4(pixel_vector, false);
		},
		
		_ => (),	// Hope and pray
	}

	return SpriteData {
		width: reader.info().width as u16,
		height: reader.info().height as u16,
		bit_depth: bit_depth,
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
		
		// Embedded palette
		let mut palette: Vec<u8> = Vec::new();
		
		if header.clut == 0x20 {
			// Get embedded palette
			let color_count: usize = 2usize.pow(header.bit_depth as u32);
			palette = vec![0; color_count * 4];
			palette.copy_from_slice(&bin_data[0x10..0x10 + color_count * 4]);
			
			// Move pointer past palette
			pointer = bin_header::HEADER_SIZE + (color_count * 4) as usize;
		}
		else {
			// Move pointer past header
			pointer = bin_header::HEADER_SIZE;
		}
		
		let mut pixels: Vec<u8> = vec![0; bin_data.len() - pointer];
		pixels.copy_from_slice(&bin_data[pointer..]);
		
		return SpriteData {
			width: header.width,
			height: header.height,
			bit_depth: header.bit_depth,
			pixels: pixels,
			palette: palette,
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
	
	// Cheers Wikipedia
	let row_size: usize = (((dib_header.bitcount as usize) * (dib_header.width as usize) + 31) / 32) * 4;
	let pixel_array_len: usize = row_size * dib_header.height.abs() as usize;
	
	let start: usize = file_header.bfOffBits as usize;
	
	let mut pixel_array: Vec<u8> = vec![0; pixel_array_len];
	pixel_array.copy_from_slice(&bmp.contents[start..start + pixel_array_len]);
	
	// Bit depth handling
	match dib_header.bitcount {
		1 => pixel_array = sprite_transform::bpp_from_1(pixel_array, false),
		2 => pixel_array = sprite_transform::bpp_from_2(pixel_array, false),
		4 => pixel_array = sprite_transform::bpp_from_4(pixel_array, false),
		8 => (),
		_ => {
			println!("Warning: Skipping BMP as its color depth is not supported ({})", dib_header.bitcount);
			println!("\tSkipped: {}", &source_file.display());
			return SpriteData::default();
		},
	}
	
	// Trim padding
	let mut pixel_vector: Vec<u8> = Vec::new();
	let u_width: usize = dib_header.width as usize;
	let u_height: usize = dib_header.height as usize;
	
	for y in (0..u_height).rev() {
		for x in 0..u_width {
			pixel_vector.push(pixel_array[y * u_width + x]);
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
	let mut palette: Vec<u8> = vec![0; color_count * 4];
	
	for color in 0..color_count {
		palette[4 * color + 0] = bmp.contents[index + (color_size * color + 2)];
		palette[4 * color + 1] = bmp.contents[index + (color_size * color + 1)];
		palette[4 * color + 2] = bmp.contents[index + (color_size * color + 0)];
		
		// 24 bit BMP input will use default +R alpha
		if color % 32 == 0 || (color as i32 - 8) % 32 == 0 && color != 8 {
			palette[4 * color + 3] = 0x00;
		}
		else {
			palette[4 * color + 3] = 0x80;
		}
	}
	
	return SpriteData {
		width: dib_header.width as u16,
		height: dib_header.height as u16,
		bit_depth: dib_header.bitcount as u16,
		pixels: pixel_vector,
		palette: palette,
	}
}