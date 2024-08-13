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