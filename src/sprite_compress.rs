use std::io::Cursor;
use std::cmp::min;
use bitstream_io::{BitReader, BitRead, BitWriter, BitWrite, BigEndian};

use crate::{
	shared_types::{SpriteData, CompressedData},
	bin_header::BinHeader,
	bit_depth,
};

const WINDOW_SIZE: usize = 512;
const TOKEN_SIZE_MAX: usize = 130;


pub fn compress(mut data: SpriteData) -> CompressedData {
	// Adapt 4 bpp pixel vector to 8 bpp
	if data.bit_depth == 4 {
		data.pixels = bit_depth::bpp_4to8(data.pixels, true);
	}

	// Loop variables
	let mut current_pixel: usize = 0;
	let mut iterations: usize = 0;
	
	// Output bit stream
	let mut compressed_stream: Vec::<u8> = Vec::new();
	let mut bit_writer = BitWriter::endian(&mut compressed_stream, BigEndian);
	
	// Iterate vector
	while current_pixel < data.pixels.len() {
		// Token window origin point
		let window_origin: usize;
		
		if current_pixel > WINDOW_SIZE {
			window_origin = current_pixel - WINDOW_SIZE;
		} else {
			window_origin = 0;
		}
		
		if current_pixel >= 4 && data.pixels.len() - current_pixel > 2 {
			let mut best_sequence_offset: usize = 0;
			let mut best_sequence_length: usize = 0;
			let mut token_size_max_local: usize = min(TOKEN_SIZE_MAX, current_pixel);
			token_size_max_local = min(token_size_max_local, data.pixels.len() - current_pixel);
			
			// New window scan, slower, better compression (matches game's)
			for window_offset in 0..510 {
				let mut sequence_length: usize = 0;
				
				while sequence_length < token_size_max_local {
					let window_index: usize = window_origin + window_offset + sequence_length;
					
					if window_index >= current_pixel {
						break;
					}
						
					if data.pixels[current_pixel + sequence_length] == data.pixels[window_index] {
						sequence_length += 1;
					} else {
						break;
					}
				}
				
				if sequence_length > best_sequence_length {
					best_sequence_length = sequence_length;
					best_sequence_offset = window_offset;
				}
				
				if sequence_length >= token_size_max_local {
					break;
				}
			}
			
			if best_sequence_length > 2 {
				let _ = bit_writer.write_bit(false);
				let _ = bit_writer.write(9, best_sequence_offset as u16);
				let _ = bit_writer.write(7, (best_sequence_length as u8) - 3);
				current_pixel += best_sequence_length;
				iterations += 1;
				continue;
			}
		}
		
		// Literal indicator
		let _ = bit_writer.write_bit(true);
		
		// Pixels
		let _ = bit_writer.write(8, data.pixels[current_pixel]);
		
		if current_pixel + 1 < data.pixels.len() {
			let _ = bit_writer.write(8, data.pixels[current_pixel + 1]);
		} else {
			let _ = bit_writer.write(8, 0u8);
		}
		
		// Increment position
		current_pixel += 2;		
		iterations += 1;
	}
	
	// Pad and close bit stream
	bit_writer.byte_align().expect("main::make_compressed_sprite() error: Could not align bitstream");
	bit_writer.into_writer();
	
	let file_byte_length: usize = compressed_stream.len() + 20;
	
	for _i in 0..(16 - file_byte_length % 16) {
		compressed_stream.push(255);
	}
	
	return CompressedData {
		iterations: iterations,
		stream: compressed_stream,
	};
}


pub fn decompress(bin_data: Vec<u8>, header: BinHeader) -> SpriteData {
	let pixel_count: usize = header.width as usize * header.height as usize;
	let mut pointer: usize = 0x10;
	let mut palette: Vec<u8> = Vec::new();
	
	// Get embedded palette
	if header.clut == 0x20 {
		let color_count: usize = 2u16.pow(header.bit_depth as u32) as usize;
		
		// Get palette
		for index in 0..color_count {
			// RGBA
			palette.push(bin_data[pointer + 4 * index + 0]);
			palette.push(bin_data[pointer + 4 * index + 1]);
			palette.push(bin_data[pointer + 4 * index + 2]);
			palette.push(bin_data[pointer + 4 * index + 3]);
		}
		
		pointer += color_count * 4;
	}
	
	// Read iterations
	let iterations: u32 = u32::from_le_bytes([
		bin_data[pointer + 0x02],
		bin_data[pointer + 0x03],
		bin_data[pointer + 0x00],
		bin_data[pointer + 0x01]
	]);
	
	// Move pointer past iterations
	pointer += 0x04;
	
	// Get byte data
	let mut byte_data: Vec<u8> = Vec::with_capacity(bin_data.len() - pointer);
	while pointer < bin_data.len() {
		byte_data.push(bin_data[pointer + 1]);
		byte_data.push(bin_data[pointer]);
		pointer += 2;
	}
	
	// Read as bit stream
	let mut bit_reader = BitReader::endian(Cursor::new(&byte_data), BigEndian);
	
	// Pixel vector
	let mut pixel_vector: Vec<u8> = Vec::new();
	
	for _i in 0..iterations {
		// Literal mode
		if bit_reader.read_bit().unwrap() == true {
			pixel_vector.push(bit_reader.read(8).unwrap());
			
			// Stray byte guard rail
			if pixel_vector.len() + 1 < pixel_count {
				pixel_vector.push(bit_reader.read(8).unwrap());
			}
		}
		
		// Token mode
		else {			
			let mut window_origin: usize = 0;
			if pixel_vector.len() > 512 {
				window_origin = pixel_vector.len() - 512;
			}
			
			let offset: usize = bit_reader.read::<u16>(9).unwrap() as usize;
			let length: usize = 3 + bit_reader.read::<u8>(7).unwrap() as usize;
			
			for pixel in 0..length {
				pixel_vector.push(pixel_vector[window_origin + offset + pixel]);
			}
		}
	}
	
	if header.bit_depth == 4 {
		pixel_vector = bit_depth::bpp_8to4(pixel_vector, true);
	}
	
	pixel_vector.resize(header.width as usize * header.height as usize, 0u8);
	
	return SpriteData {
		width: header.width,
		height: header.height,
		bit_depth: header.bit_depth,
		pixels: pixel_vector,
		palette: palette,
	};
}