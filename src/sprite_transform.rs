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