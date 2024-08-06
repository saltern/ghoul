use std::io::Cursor;
use bitstream_io::{BitReader, BitRead, BitWriter, BitWrite, BigEndian};

use crate::shared_types::{
	SpriteData,
	CompressedData,
};

const WINDOW_SIZE: usize = 512;
const TOKEN_SIZE_MAX: usize = 130;


pub fn compress(data: SpriteData) -> CompressedData {
	// Loop variables
	let mut current_pixel: usize = 0;
	let mut iterations: usize = 0;
	
	// Output bit stream
	let mut compressed_stream: Vec::<u8> = Vec::new();
	let mut bit_writer = BitWriter::endian(&mut compressed_stream, BigEndian);
	
	// Iterate vector
	while current_pixel < data.pixels.len() {
		// Token window origin point
		let mut window_origin: usize = 0;
		if current_pixel > WINDOW_SIZE {
			window_origin = current_pixel - WINDOW_SIZE;
		}
		
		// Used later
		let mut token_write: bool = false;
		
		// Token window
		if current_pixel >= 4 {
			// Record keeping
			let mut sequence_length: usize = 0;
			let mut best_sequence_offset: usize = 0;
			let mut best_sequence_length: usize = 0;
			
			// Window scan
			for window_element in window_origin..current_pixel + 1 {
				// Index of sequence's current pixel
				let sequence_element: usize = current_pixel + sequence_length;
				
				// Conditions that should end a sequence
				let mut end_sequence: bool = sequence_length >= TOKEN_SIZE_MAX; // Maximum sequence length reached
				end_sequence = end_sequence || window_element >= current_pixel; // Last element of window reached
				end_sequence = end_sequence || sequence_element >= data.pixels.len(); // End of sprite reached
				
				// Pixel match check
				if !end_sequence {
					if data.pixels[sequence_element] == data.pixels[window_element] {
						sequence_length += 1;
					}
					
					else {
						end_sequence = true;
					}
				}
				
				if end_sequence {
					// Register sequence if better
					if sequence_length > best_sequence_length {
						best_sequence_length = sequence_length;
						best_sequence_offset = window_element - window_origin - sequence_length;
					}
					
					// Reset sequence
					sequence_length = 0;
				}
			}
			
			// Write token if long enough
			if best_sequence_length > 2 {
				// Token indicator
				let _ = bit_writer.write_bit(false);
				
				// Token offset
				let _ = bit_writer.write(9, best_sequence_offset as u16);
				
				// Token length
				let _ = bit_writer.write(7, (best_sequence_length as u8) - 3);
				
				// Increment position
				current_pixel += best_sequence_length;
				token_write = true;
			}
		}
		
		if !token_write {
			// Literal indicator
			let _ = bit_writer.write_bit(true);
			
			// Pixels
			let _ = bit_writer.write(8, data.pixels[current_pixel]);
			
			if current_pixel + 1 < data.pixels.len() {
				let _ = bit_writer.write(8, data.pixels[current_pixel + 1]);
			}
			
			else {
				let _ = bit_writer.write(8, 0u8);
			}
			
			// Increment position
			current_pixel += 2;
		}
		
		iterations += 1
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


pub fn decompress(bin_data: Vec<u8>) -> SpriteData {
	let dimensions: (u16, u16) = (
		u16::from_le_bytes([bin_data[6], bin_data[7]]),
		u16::from_le_bytes([bin_data[8], bin_data[9]]));
	
	let pixel_count: usize = (dimensions.0 as usize) * (dimensions.1 as usize);
	
	let iterations: u32 = u32::from_le_bytes([
		bin_data[18], bin_data[19], bin_data[16], bin_data[17]]);
	
	// Initialize byte data...
	let mut byte_data: Vec<u8> = Vec::new();
	let mut pointer: usize = 20;	// Skip header and iterations
	
	// Get byte data
	while pointer < bin_data.len() {
		byte_data.push(bin_data[pointer + 1]);
		byte_data.push(bin_data[pointer]);
		pointer += 2;
	}
	
	// Read as bit stream
	let mut bit_reader = BitReader::endian(Cursor::new(&byte_data), BigEndian);
	
	// Pixel vector
	let mut pixel_vector: Vec<u8> = Vec::new();
	let mut current_pixel: usize = 0;
	
	for _i in 0..iterations {
		// Literal mode
		if bit_reader.read_bit().unwrap() == true {
			pixel_vector.push(bit_reader.read(8).unwrap());
			
			// Guard against stray pixels
			if current_pixel + 1 < pixel_count {
				pixel_vector.push(bit_reader.read(8).unwrap());
			}
			
			current_pixel += 2;
		}
		
		// Token mode
		else {			
			let mut window_origin: usize = 0;
			if current_pixel > 512 {
				window_origin = current_pixel - 512;
			}
			
			let offset: usize = bit_reader.read::<u16>(9).unwrap() as usize;
			let length: usize = bit_reader.read::<u8>(7).unwrap() as usize;
			
			for pixel in 0..length + 3 {
				pixel_vector.push(pixel_vector[window_origin + offset + pixel]);
			}
			
			current_pixel += length + 3;
		}
	}
	
	return SpriteData {
		width: dimensions.0,
		height: dimensions.1,
		pixels: pixel_vector,
	};
}