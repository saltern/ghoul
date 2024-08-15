pub fn bpp_4to8(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
	let mut output_pixels: Vec<u8> = Vec::new();
	let mut index: usize = 0;
	
	// When compressing a 4 bpp sprite, two pixels become one byte. Chop two adjacent
	// pixels down to 4 bits and combine them into one 8-bit value.
	while index < input_pixels.len() {
		if index + 1 < input_pixels.len() {
			output_pixels.push(
				input_pixels[index + flip as usize] << 4 |
				input_pixels[index + !flip as usize] & 0xF
			);
		}
		
		else {
			if flip {
				output_pixels.push(0x00 | input_pixels[index] << 4);
			}
			else {
				output_pixels.push(input_pixels[index] << 4 | 0x00);
			}
		}
		index += 2;
	}
	
	return output_pixels;
}


pub fn bpp_8to4(input_pixels: Vec<u8>, flip: bool) -> Vec<u8> {
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