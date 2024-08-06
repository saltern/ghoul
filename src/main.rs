use std::env;
use std::path::PathBuf;

pub mod shared_types;
pub mod param_validator;
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
		process_directory(parameters);
	}
	
	else {
		process_file(parameters);
	}
}


pub fn help_message() {
	println!();
	println!("Tool for handling GGXX AC+R sprites.");
	println!("Can convert and reindex PNG-, RAW-, and BIN-format sprites.");
	println!();
	println!("Usage:");
	println!("    ghoul -i <input file>");
	println!();
	println!("To process full directories, use an asterisk as the input file name (e.g. 'path/*.png').");
	println!();
	println!("    Available parameters:");
	println!("        -f <format>       Convert sprites (formats: 'png', 'raw', 'bin')");
	println!("        -o <path>         Set output path, defaults to the current directory if not specified");
	println!("        -p <pal file>     If output format is 'png', color sprites using this .act palette");
	println!("        -r                Reindex output sprites");
	println!("        -u                Output uncompressed sprites (BIN only)");
	println!("        -w                Overwrite pre-existing files");
	println!("        -l                List processed files");
	println!();
	println!("When an output path is not specified, the current working directory will be used.");
}


fn process_file(parameters: Parameters) {
	if parameters.verbose {
		match parameters.source_path.file_name() {
			Some(name) => println!("Processing '{}'...", name.to_str().unwrap()),
			_ => (),
		}
	}
	
	let mut data: SpriteData = SpriteData::default();
	
	match parameters.source_format {
		SpriteFormat::PNG => data = sprite_get::get_png(&parameters.source_path),
		SpriteFormat::RAW => data = sprite_get::get_raw(&parameters.source_path),
		SpriteFormat::BIN => data = sprite_get::get_bin(&parameters.source_path),
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
		SpriteFormat::PNG => sprite_make::make_png(parameters.source_path, parameters.target_path, data, parameters.palette_file, parameters.overwrite),
		SpriteFormat::RAW => sprite_make::make_raw(parameters.source_path, parameters.target_path, data, parameters.overwrite),
		SpriteFormat::BIN => sprite_make::make_bin(parameters.source_path, parameters.target_path, data, parameters.uncompressed, parameters.overwrite),
		_ => println!("main::process_file() error: Invalid target format provided"),
	}
}


fn process_directory(parameters: Parameters) {
	let mut items_processed: usize = 0;

	for item in parameters.source_path.read_dir().expect("main::process_directory() error: Could not read source path") {
		let this_path: PathBuf = item.unwrap().path();
		
		if !this_path.is_file() {
			continue;
		}
		
		let mut type_match: bool = false;
		
		match this_path.extension() {
			Some(os_str) => match os_str.to_ascii_lowercase().to_str() {
				Some("png") => if parameters.source_format == SpriteFormat::PNG {
					type_match = true;
				},
				
				Some("raw") => if parameters.source_format == SpriteFormat::RAW {
					type_match = true;
				},
				
				Some("bin") => if parameters.source_format == SpriteFormat::BIN {
					type_match = true;
				},
				
				_ => (),
			}
			_ => (),
		}
		
		if type_match {
			let mut new_params: Parameters = parameters.clone();
			new_params.source_path = this_path;
			process_file(new_params);
			items_processed += 1;
		}
	}
	
	println!("Processed {} sprites", items_processed);
}