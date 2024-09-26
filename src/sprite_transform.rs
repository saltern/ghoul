use std::cmp;


pub fn transform_index(mut value: u8) -> u8 {
	// Divide the currently read byte by 8.
	// - If remainder + 2 can be evenly divided by 4, output is byte value - 8
	// - If remainder + 3 can be evenly divided by 4, output is byte value + 8
	// The original value is passed through otherwise
	
	if ((value / 8) + 2) % 4 == 0 {
		value -= 8;
	}
	
	else if ((value / 8) + 3) % 4 == 0 {
		value += 8
	}
	
	return value;
}


pub fn indexed_as_rgb(input_pixels: Vec<u8>, palette: &Vec<u8>) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	
	for pixel in 0..input_pixels.len() {
		output_pixels.push(palette[4 * input_pixels[pixel] as usize]);
	}
	
	return output_pixels;
}


// Works, but not currently used.
// pub fn bpp_to_1(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
	// let mut output_pixels: Vec<u8> = Vec::new();
	// let mut index: usize = 0;
	
	// while index < input_pixels.len() {
		// let process_count: usize = min(8, input_pixels.len() - index);
		// let mut byte: u8 = 0;
		
		// for pixel in 0..process_count {
			// let bit: u8 = (input_pixels[index + pixel] > 0) as u8;
			// byte = byte | bit << (7 - pixel);
		// }
		
		// if flip {
			// byte = byte.reverse_bits();
		// }
		
		// output_pixels.push(byte);
		// index += 8;
	// }
	
	// return output_pixels;
// }


pub fn bpp_from_1(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	
	for index in 0..input_pixels.len() {
		let mut byte: u8 = input_pixels[index];
		
		if !flip {
			byte = byte.reverse_bits();
		}
		
		for shift in 0..8 {
			output_pixels.push((byte >> shift) & 0x1);
		}
	}
	
	return output_pixels;
}


// Works, but not currently used.
// pub fn bpp_to_2(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
	// let mut output_pixels: Vec<u8> = Vec::new();
	// let mut index: usize = 0;
	
	// while index < input_pixels.len() {
		// let process_count: usize = min(4, input_pixels.len() - index);
		// let mut byte_pixels: Vec<u8> = Vec::new();
		// let mut byte: u8 = 0;
		
		// for pixel in 0..process_count {
			// byte_pixels.push(input_pixels[index + pixel]);
		// }
		
		// if !flip {
			// byte_pixels.reverse();
		// }
		
		// for pixel in 0..byte_pixels.len() {
			// byte = byte | byte_pixels[pixel] << 2 * pixel;
		// }
		
		// output_pixels.push(byte);
		// index += 4;
	// }
	
	// return output_pixels;
// }


pub fn bpp_from_2(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	
	if flip {
		for index in 0..input_pixels.len() {
			output_pixels.push((input_pixels[index] >> 0) & 0x3);
			output_pixels.push((input_pixels[index] >> 2) & 0x3);
			output_pixels.push((input_pixels[index] >> 4) & 0x3);
			output_pixels.push((input_pixels[index] >> 6) & 0x3);
		}
	}
	else {
		for index in 0..input_pixels.len() {
			output_pixels.push((input_pixels[index] >> 6) & 0x3);
			output_pixels.push((input_pixels[index] >> 4) & 0x3);
			output_pixels.push((input_pixels[index] >> 2) & 0x3);
			output_pixels.push((input_pixels[index] >> 0) & 0x3);
		}
	}
	
	return output_pixels;
}
	

pub fn bpp_to_4(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	let mut index: usize = 0;
	
	// When compressing a 4 bpp sprite, two pixels become one byte. Chop two adjacent
	// pixels down to 4 bits and combine them into one 8-bit value.
	while index < input_pixels.len() {
		if index + 1 < input_pixels.len() {
			output_pixels.push(
				cmp::min(input_pixels[index + flip as usize], 0xF) << 4 |
				cmp::min(input_pixels[index + !flip as usize], 0xF) & 0xF
			);
		}
		
		else {
			if flip {
				output_pixels.push(cmp::min(input_pixels[index], 0xF) & 0xF);
			}
			
			else {
				output_pixels.push(cmp::min(input_pixels[index], 0xF) << 4);
			}
		}
		
		index += 2;
	}
	
	return output_pixels;
}


pub fn bpp_from_4(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	
	// When decompressing a 4 bpp sprite, the resulting pixel vector will contain
	// two pixels per byte (0000-0000). Separate and push them to output.
	if flip {
		for index in 0..input_pixels.len() {
			output_pixels.push(input_pixels[index] & 0xF);
			output_pixels.push(input_pixels[index] >> 4);
		}
	}
	
	else {
		for index in 0..input_pixels.len() {
			output_pixels.push(input_pixels[index] >> 4);
			output_pixels.push(input_pixels[index] & 0xF);
		}
	}
	
	return output_pixels;
}


pub fn align_to_4(input_pixels: Vec<u8>, height: usize) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	
	let width: usize = input_pixels.len() / height;
	let padding: usize = width % 2;
	
	for y in 0..height {
		for x in 0..width {
			output_pixels.push(input_pixels[y * width + x]);
		}
		
		for _x in 0..padding {
			output_pixels.push(0x00);
		}
	}
	
	return output_pixels;
}


pub fn trim_padding(input_pixels: Vec<u8>, width: usize, height: usize) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	let row_width: usize = input_pixels.len() / height;
	
	for y in 0..height {
		for x in 0..width {
			output_pixels.push(input_pixels[y * row_width + x]);
		}
	}
	
	return output_pixels;
}