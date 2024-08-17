use std::io::{Write, BufWriter};
use std::fs::File;

use crate::{
	PathBuf,
	Parameters,
	SpriteData,
	SpriteFormat,
	shared_types::CompressedData,
	bin_header,
	sprite_compress,
	sprite_transform,
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


fn bmp_header(width: u16, height: u16, bit_depth: u16) -> Vec<u8> {
	let mut bmp_data: Vec<u8> = Vec::new();
	
	// BITMAPFILEHEADER
	// 2 bytes, "BM"
	bmp_data.push(0x42);
	bmp_data.push(0x4d);
	
	// 4 bytes, size of the bitmap in bytes
	// 14 bytes - BITMAPFILEHEADER
	// 12 bytes - DIBHEADER of type BITMAPCOREHEADER
	let header_length: u32 = 14 + 12 + 2u32.pow(bit_depth as u32) * 3;
	let bmp_file_size: [u8; 4] = (header_length + (width + width % 4) as u32 * height as u32).to_le_bytes();
	for byte in 0..4 {
		bmp_data.push(bmp_file_size[byte]);
	}
	
	// 2 bytes each for bfReserved1 and 2
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	bmp_data.push(0x00);
	
	// 4 bytes, offset to pixel array
	bmp_data.push((header_length & 0xFF) as u8);
	bmp_data.push((header_length >> 8) as u8);
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
	match bit_depth {
		1 => bmp_data.push(0x01),
		2 => bmp_data.push(0x02),
		4 => bmp_data.push(0x04),
		8 => bmp_data.push(0x08),
		
		// Theoretically shouldn't happen
		_ => panic!("sprite_make::bmp_header() error: Invalid color depth!"),
	}

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
	
	// 4 bpp handling
	let working_pixels: Vec<u8>;
	
	match data.bit_depth {
		// 1 and 2 bpp not currently in use
		// 1 => {
			// working_pixels = sprite_transform::bpp_to_1(data.pixels, false);
			// encoder.set_depth(png::BitDepth::One);
		// },

		// 2 => {
			// working_pixels = sprite_transform::bpp_to_2(data.pixels, false);
			// encoder.set_depth(png::BitDepth::Two);
		// },
		
		4 => {
			working_pixels = sprite_transform::bpp_to_4(data.pixels, false);
			encoder.set_depth(png::BitDepth::Four);
		},
		
		8 => {
			working_pixels = data.pixels;
			encoder.set_depth(png::BitDepth::Eight);
		},
		
		// Shouldn't happen
		_ => panic!("sprite_make::make_png() error: Invalid bit depth"),
	}
	
	// encoder.set_color(png::ColorType::Grayscale);
	encoder.set_color(png::ColorType::Indexed);
	
	// Palette
	let color_count: usize = 2usize.pow(data.bit_depth as u32);
	let mut rgb_palette: Vec<u8> = Vec::with_capacity(color_count * 3);
	let mut transparency: Vec<u8> = vec![0xFF; color_count];
	
	// Color pal
	if !data.palette.is_empty() {
		for color in 0..color_count {
			rgb_palette.push(data.palette[4 * color + 0]);
			rgb_palette.push(data.palette[4 * color + 1]);
			rgb_palette.push(data.palette[4 * color + 2]);
			transparency[color] = data.palette[4 * color + 3];
		}
		// encoder.set_color(png::ColorType::Indexed);
	}
	
	// Grayscale pal
	else {
		for color in 0..color_count {
			rgb_palette.push(color as u8);
			rgb_palette.push(color as u8);
			rgb_palette.push(color as u8);
			transparency[color] = 0xFF;
		}
	}
			
	encoder.set_palette(rgb_palette);
	encoder.set_trns(transparency);
	
	let mut writer = encoder.write_header().expect("sprite_make::make_png() error: Could not write PNG header");
	writer.write_image_data(&working_pixels).unwrap();
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


pub fn make_bin(parameters: Parameters, mut data: SpriteData) {
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
	
	let clut: u16;
	if data.palette.is_empty() {
		clut = 0x0000;
	}
	else {
		clut = 0x0020;
	}
	
	// Header
	// Ugly as sin, I should change this later
	let header_vector: Vec<u8> = bin_header::get_bytes(bin_header::BinHeader {
		compressed: !parameters.uncompressed,
		clut: clut,
		bit_depth: data.bit_depth,
		width: data.width,
		height: data.height,
		tw: 0x0,
		th: 0x0,
		hash: 0x0,
	});
	
	let _ = buffer.write_all(&header_vector);
	
	// clut
	let _ = buffer.write_all(&data.palette);
	
	// Uncompressed mode
	if parameters.uncompressed {
		if data.bit_depth == 4 {
			data.pixels = sprite_transform::bpp_to_4(data.pixels, true);
		}
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
	let header: Vec<u8> = bmp_header(data.width, data.height, data.bit_depth);
	
	// Color table
	let mut color_table: Vec<u8> = Vec::with_capacity(768);
	let color_count: usize = 2usize.pow(data.bit_depth as u32);
	
	// Grayscale
	if data.palette.is_empty() {
		for color in 0..color_count {
			color_table.push(color as u8);
			color_table.push(color as u8);
			color_table.push(color as u8);
		}
	}
	
	// Palette (no alpha)
	else {
		for color in 0..color_count {
			color_table.push(data.palette[4 * color + 2]);
			color_table.push(data.palette[4 * color + 1]);
			color_table.push(data.palette[4 * color + 0]);
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
	
	let byte_vector: Vec<u8>;
	
	match data.bit_depth {
		// 1 and 2 bpp not currently in use
		// 1 => byte_vector = sprite_transform::bpp_to_1(data.pixels, false),
		// 2 => byte_vector = sprite_transform::bpp_to_2(data.pixels, false),
		4 => byte_vector = sprite_transform::bpp_to_4(data.pixels, false),
		8 => byte_vector = data.pixels,
		// Shouldn't happen
		_ => panic!("sprite_make::make_bmp() error: Invalid bit depth"),
	}
	
	// Cheers Wikipedia
	let row_length: usize = (((data.bit_depth * data.width + 31) / 32) * 4) as usize;
	let byte_width: usize = byte_vector.len() / data.height as usize;
	let padding: usize = row_length - byte_width;
	
	// Upside-down write with padding
	for y in (0..data.height as usize).rev() {
		let row_start: usize = y * byte_width;
		let _ = buffer.write_all(&byte_vector[row_start..row_start + byte_width]);
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