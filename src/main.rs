use std::env;
use std::thread;
use std::fs;
use std::fs::ReadDir;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Instant;

pub mod shared_types;
pub mod param_validator;
pub mod bin_header;
pub mod sprite_get;
pub mod sprite_make;
pub mod sprite_compress;
pub mod sprite_transform;

use crate::shared_types::{
	Parameters,
	SpriteData,
	SpriteFormat
};


pub fn main() {
	let args: Vec<String> = env::args().collect();
	let args_length: usize = args.len();
	
	// Parse arguments.
	// At a minimum, will need 4 parameters ('ghoul', '-i', file name, operation)
	if args_length < 4 {
		help_message();
		return;
	}
	
	let opt_parameters: Option<Parameters> = param_validator::validate(args_length, args);
	
	match opt_parameters {
		None => {
			println!("Call 'ghoul' by itself for help.");
			return;
		},
		
		_ => (),
	}
	
	let parameters: Parameters = opt_parameters.unwrap();
	
	if parameters.forced_bit_depth {
		println!("Note: Changing a sprite's color depth could produce incorrect results ingame.");
	}
	
	println!("Working...");
	let instant = Instant::now();
	
	if parameters.directory_mode {
		process_directory(parameters);
	}
	
	else {
		process_file(parameters);
		print!("Processed 1 sprite");
	}
	
	println!(" in {}ms.", instant.elapsed().as_millis());
}


pub fn help_message() {
	println!();
	println!("Tool for handling GGXX AC+R sprites.");
	println!("Can convert and reindex PNG-, RAW-, BIN-, and BMP-format sprites.");
	println!();
	println!("Usage:");
	println!("    ghoul -i <input file> [-f format] [-o output] [-p/-c] [-4/-8] [-r] [-u] [-w] [-l]");
	println!();
	println!("To process full directories, use an asterisk as the input file name (e.g. 'path/*.png').");
	println!();
	println!("Available parameters:");
	println!("    -f or -format  <format>      Convert sprites (formats: 'png', 'raw', 'bin', 'bmp')");
	println!("    -o or -output  <path>        Set output path, defaults to the current directory if not specified");
	println!("    -p or -palette <pal file>    Color output sprite using this .act palette (except RAWs)");
	println!("    -c or -palcopy               Copy source sprite's palette to output sprite (except RAWs, overrides -p)");
	println!("    -4 or -force-4bpp            Force output to 4-bit color depth (except RAWs)");
	println!("    -8 or -force-8bpp            Force output to 8-bit color depth (except RAWs)");
	println!("    -rgb or -as-rgb              Force input to be treated as RGB (except grayscale)");
	println!("    -r or -reindex               Reindex output sprites");
	println!("    -u or -uncompressed          Output uncompressed sprites (BIN only)");
	println!("    -w or -overwrite             Overwrite pre-existing files");
	println!("    -l or -list                  List processed files");
	println!();
	println!("When an output path is not specified, the current working directory will be used.");
}


fn process_file(parameters: Parameters) {
	if parameters.verbose {
		match parameters.source_path.file_name() {
			Some(name) => println!("Processing '{}'", name.to_str().unwrap()),
			_ => (),
		}
	}
	
	let mut data: SpriteData = SpriteData::default();
	
	match parameters.source_format {
		SpriteFormat::PNG => data = sprite_get::get_png(&parameters.source_path),
		SpriteFormat::RAW => data = sprite_get::get_raw(&parameters.source_path),
		SpriteFormat::BIN => data = sprite_get::get_bin(&parameters.source_path),
		SpriteFormat::BMP => data = sprite_get::get_bmp(&parameters.source_path),
		_ => println!("main::process_file() error: Invalid source format provided"),
	}
	
	if data.width == 0 || data.height == 0 {
		return;
	}
	
	// Reindex
	if parameters.reindex {
		for index in 0..data.pixels.len() {
			data.pixels[index] = sprite_transform::transform_index(data.pixels[index]);
		}
	}
	
	// -as-rgb
	if !data.palette.is_empty() && parameters.as_rgb {
		data.pixels = sprite_transform::indexed_as_rgb(data.pixels, &data.palette);
	}
	
	// -force-4bpp / -force-8bpp
	if parameters.forced_bit_depth {
		data.bit_depth = parameters.bit_depth as u16;
	}
	else {
		data.bit_depth = std::cmp::max(data.bit_depth, 4);
	}
	
	let mut temp_palette: Vec<u8> = Vec::new();
	let color_count: usize = 2usize.pow(data.bit_depth as u32);
	let alpha_processing: bool;
	
	// Prioritize -palcopy
	if parameters.palette_transfer {
		alpha_processing = false;
		
		if data.palette.is_empty() {
			println!("Warning: Will not -palcopy as source contains no palette");
			println!("\tFile: {}", parameters.source_path.display());
		}
		
		else {
			temp_palette = data.palette;
			
			// Expand palette
			if temp_palette.len() < 4 * color_count {
				for index in 0..color_count - (temp_palette.len() / 4) {
					// RGB
					temp_palette.push(0x00);
					temp_palette.push(0x00);
					temp_palette.push(0x00);
				
					// Default alpha
					if (index / 16) % 2 == 0 && index % 8 == 0 && index != 8 {
						temp_palette.push(0x00);
					}
					
					else {
						temp_palette.push(0x80);
					}
				}
			}
			
			// Truncate palette
			else {
				temp_palette.resize(color_count * 4, 0u8);
			}
		}
	}
	
	// Else move on to -palette
	else if !parameters.palette_file.as_os_str().is_empty() {
		match fs::read(&parameters.palette_file) {
			Ok(data) => {
				// Currently treating all input palettes as RGB, this might change	
				alpha_processing = true;
							
				for index in 0..data.len() {
					// Actual color
					temp_palette.push(data[index]);
					
					// Alpha, modified later
					if (index + 1) % 3 == 0 {
						temp_palette.push(0x80);
					}
				}
			},
			
			_ => {
				alpha_processing = false;
				println!("main::process_file() error: Could not read source palette file, ignoring");
			},
		}
	}
	
	// No palette
	else {
		alpha_processing = false;
	}
	
	// Process palette alpha
	// Applies default values in case no alpha data was present in source palette
	if alpha_processing {
		// Expand or truncate to 16 or 256 colors with alpha
		temp_palette.resize(color_count * 4, 0u8);
		
		for index in 0..color_count {
			if (index / 16) % 2 == 0 && index % 8 == 0 && index != 8 {
				temp_palette[4 * index + 3] = 0x00;
			}
			else {
				temp_palette[4 * index + 3] = 0x80;
			}
		}
	}
	
	// Pass result to data.
	data.palette = temp_palette;
	
	match parameters.target_format {
		SpriteFormat::PNG => sprite_make::make_png(parameters, data),
		SpriteFormat::RAW => sprite_make::make_raw(parameters, data),
		SpriteFormat::BIN => sprite_make::make_bin(parameters, data),
		SpriteFormat::BMP => sprite_make::make_bmp(parameters, data),
		_ => println!("main::process_file() error: Invalid target format provided"),
	}
}


fn type_matches(extension: Option<&OsStr>, format: SpriteFormat) -> bool {
	match extension {
		Some(os_str) => match os_str.to_ascii_lowercase().to_str() {
			Some("png") => return format == SpriteFormat::PNG,
			Some("raw") => return format == SpriteFormat::RAW,
			Some("bin") => return format == SpriteFormat::BIN,
			Some("bmp") => return format == SpriteFormat::BMP,
			_ => return false,
		},
		_ => return false,
	}
}


fn process_directory_thread(mut parameters: Parameters, offset: usize) -> usize {
	let mut items_processed: usize = 0;
	let mut directory_items: ReadDir = parameters.source_path.read_dir().expect(
		"main::process_directory_thread() error: Could not read source path");

	for _i in 0..offset {
		directory_items.next();
	}

	loop {
		match directory_items.next() {
			Some(item) => {
				let this_path: PathBuf = item.unwrap().path();
				
				if type_matches(this_path.extension(), parameters.source_format) {
					parameters.source_path = this_path;
					process_file(parameters.clone());
					items_processed += 1;
				}
			},
			None => return items_processed,
		}
		directory_items.next();
	}
}


// Experimental multithreading
fn process_directory(parameters: Parameters) {
	let params_t2: Parameters = parameters.clone();
	
	let handle = thread::spawn(move || process_directory_thread(params_t2, 1));
	let item_count: usize = process_directory_thread(parameters, 0);
	
	// Wait for both threads to be done
	let item_count_t2: usize = handle.join().unwrap();
	
	print!("Processed {} sprites", item_count + item_count_t2);
	// print!("Processed {} sprites", item_count);
}