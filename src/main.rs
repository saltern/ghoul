use std::env;
use std::thread;
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
	
	if parameters.directory_mode {
		println!("Working...");
		let instant = Instant::now();
		process_directory(parameters);
		println!(" in {}ms.", instant.elapsed().as_millis());
	}
	
	else {
		process_file(parameters);
	}
}


pub fn help_message() {
	println!();
	println!("Tool for handling GGXX AC+R sprites.");
	println!("Can convert and reindex PNG-, RAW-, BIN-, and BMP-format sprites.");
	println!();
	println!("Usage:");
	println!("    ghoul -i <input file> [-f format] [-o output] [-p/-c] [-r] [-u] [-w] [-l]");
	println!();
	println!("To process full directories, use an asterisk as the input file name (e.g. 'path/*.png').");
	println!();
	println!("Available parameters:");
	println!("    -f or -format  <format>      Convert sprites (formats: 'png', 'raw', 'bin', 'bmp')");
	println!("    -o or -output  <path>        Set output path, defaults to the current directory if not specified");
	println!("    -p or -palette <pal file>    If output format is 'png', 'bmp', or 'bin', color sprites using this .act palette");
	println!("    -c or -palcopy               Copy source palette to output (PNG and BMP only)");
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
}