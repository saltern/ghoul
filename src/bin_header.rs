pub const HEADER_SIZE: usize = 16;

pub struct BinHeader {
	pub compressed: bool,
	pub clut: u16,
	pub bit_depth: u16,
	pub width: u16,
	pub height: u16,
	pub tw: u16,
	pub th: u16,
	pub hash: u16,
}


pub fn get_header(data: Vec<u8>) -> BinHeader {	
	return BinHeader {
		compressed: data[0] == 1,
		
		clut: u16::from_le_bytes([
			data[0x02], data[0x03]
		]),
			
		bit_depth: u16::from_le_bytes([
			data[0x04], data[0x05]
		]),
		
		width: u16::from_le_bytes([
			data[0x06], data[0x07]
		]),
		
		height: u16::from_le_bytes([
			data[0x08], data[0x09]
		]),
		
		tw: u16::from_le_bytes([
			data[0x0A], data[0x0B]
		]),
		
		th: u16::from_le_bytes([
			data[0x0C], data[0x0D]
		]),
		
		hash: u16::from_le_bytes([
			data[0x0E], data[0x0F]
		]),
	}
}


pub fn get_bytes(header: BinHeader) -> Vec<u8> {
	let mut return_vector: Vec<u8> = Vec::with_capacity(0x10);
	
	// mode (compressed/uncompressed)
	return_vector.push(header.compressed as u8);
	return_vector.push(0x00);
	
	// clut (embedded palette)
	return_vector.extend_from_slice(&header.clut.to_le_bytes());
	
	// pix (bit depth)
	return_vector.extend_from_slice(&header.bit_depth.to_le_bytes());
	
	// width
	return_vector.extend_from_slice(&header.width.to_le_bytes());
	
	// height
	return_vector.extend_from_slice(&header.height.to_le_bytes());
	
	// tw (unknown)
	return_vector.extend_from_slice(&header.tw.to_le_bytes());
	
	// th (unknown)
	return_vector.extend_from_slice(&header.th.to_le_bytes());
	
	// hash (generation method unknown, doesn't affect result)
	return_vector.extend_from_slice(&header.hash.to_le_bytes());
	
	return return_vector;
}