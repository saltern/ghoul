use std::fs;
use std::ffi::OsStr;

use crate::{
	PathBuf,
	Parameters,
	SpriteFormat
};

enum ArgumentType {
	NONE,
	INPUT,
	OUTPUT,
	FORMAT,
	PALETTE,
}

pub fn validate(arg_count: usize, args: Vec<String>) -> Option<Parameters> {
	let mut source_file_name: &str = "";
	let mut source_palette: &str = "";
	let mut target_format: SpriteFormat = SpriteFormat::NONE;
	let mut target_path: PathBuf = PathBuf::from(".");
	let mut palette_transfer: bool = false;
	let mut forced_bit_depth: bool = false;
	let mut bit_depth: usize = 8;
	let mut uncompressed: bool = false;
	let mut reindex: bool = false;
	let mut verbose: bool = false;
	let mut overwrite: bool = false;
	
	let mut next_arg: ArgumentType = ArgumentType::NONE;
	
	// Skip executable name and mode
	for argument in 1..arg_count {
		let this_argument: &str = &args[argument].to_string();
	
		match next_arg {
			ArgumentType::INPUT => {
				source_file_name = &args[argument];
				next_arg = ArgumentType::NONE;
				continue;
			},
			
			ArgumentType::OUTPUT => {
				let this_path: &String = &args[argument];
				
				match PathBuf::from(this_path).try_exists() {
					// Already exists, do nothing
					Ok(true) => (),
					
					// Doesn't exist, create
					Ok(false) => {
						match fs::create_dir(&args[argument]) {
							// Creation successful, do nothing
							Ok(()) => (),
							
							// Creation failed
							_ => {
								println!("param_validator::validate() error: Could not create output directory, aborting");
								return None;
							},
						}
					},
					
					// Invalid path provided
					_ => {
						println!("param_validator::validate() error: Could not validate output directory, aborting");
						println!("    -> Double-check your output path for any invalid characters.");
						return None;
					}
				}
				
				target_path = PathBuf::from(format!("{}/", &args[argument]));
				next_arg = ArgumentType::NONE;
				continue;
			},
			
			ArgumentType::FORMAT => {
				match &this_argument.to_lowercase() as &str {
					"png" => {
						target_format = SpriteFormat::PNG;
						next_arg = ArgumentType::NONE;
						continue;
					},
					
					"raw" => {
						target_format = SpriteFormat::RAW;
						next_arg = ArgumentType::NONE;
						continue;
					},
					
					"bin" => {
						target_format = SpriteFormat::BIN;
						next_arg = ArgumentType::NONE;
						continue;
					},
					
					"bmp" => {
						target_format = SpriteFormat::BMP;
						next_arg = ArgumentType::NONE;
						continue;
					},
					
					_ => {
						println!("Unsupported output format '{}'. Supported formats: 'png', 'raw', 'bin'.", argument);
						return None;
					},
				}
			},
			
			ArgumentType::PALETTE => {
				source_palette = &args[argument];
				next_arg = ArgumentType::NONE;
				continue;
			},
			
			_ => (),
		}
			
		next_arg = ArgumentType::NONE;
	
		match &this_argument.to_lowercase() as &str {
			"-i" | "-input" => next_arg = ArgumentType::INPUT,
			"-o" | "-output" => next_arg = ArgumentType::OUTPUT,
			"-f" | "-format" => next_arg = ArgumentType::FORMAT,
			"-p" | "-palette" => next_arg = ArgumentType::PALETTE,
			"-c" | "-palcopy" => palette_transfer = true,
			"-r" | "-reindex" => reindex = true,
			"-l" | "-list" => verbose = true,
			"-u" | "-uncompressed" => uncompressed = true,
			"-w" | "-overwrite" => overwrite = true,
			"-4" | "-force-4bpp" => {
				forced_bit_depth = true;
				bit_depth = 4;
			},
			"-8" | "-force-8bpp" => {
				forced_bit_depth = true;
				bit_depth = 8;
			},
			_ => (),
		}
	}
	
	// Insufficient parameters
	// No source file
	if source_file_name == "" {
		println!("No source file was specified. Use '-i <source file>'.");
		return None;
	}
	
	// Get source data... for directory processing
	let source_pathbuf: PathBuf = PathBuf::from(source_file_name);
	let mut source_base_path: PathBuf = source_pathbuf.clone();
	source_base_path.set_file_name("");
	
	// Dumb hack to allow '*.png' as input without needing to do './*.png'
	if source_base_path == PathBuf::from("") {
		source_base_path = PathBuf::from(".");
	}
	
	// Set source format
	let source_file_stem: &OsStr = source_pathbuf.file_stem().unwrap();
	let source_extension: &str;
	
	match source_pathbuf.extension() {
		Some(os_str) => source_extension = os_str.to_str().unwrap(),
		_ => {
			println!("Source file format wasn't specified ('.png', '.raw', '.bin').");
			return None;
		}
	}
	
	let source_format: SpriteFormat;
	
	match &source_extension.to_lowercase() as &str {
		"png" => source_format = SpriteFormat::PNG,
		"raw" => source_format = SpriteFormat::RAW,
		"bin" => source_format = SpriteFormat::BIN,
		"bmp" => source_format = SpriteFormat::BMP,
		_ => {
			println!("Unsupported source format '{}'. Supported formats: 'png', 'raw', 'bin', 'bmp'.", source_extension);
			return None;
		},
	}
	
	if target_format == SpriteFormat::NONE {
		target_format = source_format.clone();
	}
	
	// Validate palette
	let mut palette_pathbuf: PathBuf = PathBuf::from(source_palette);
	
	if source_palette != "" {
		match palette_pathbuf.try_exists() {
			Ok(false) => {
				println!("Could not locate specified palette file, ignoring.");
				palette_pathbuf.clear();
			},
			
			Ok(true) => {
				match target_format {
					SpriteFormat::RAW => {
						println!("A palette has been specified but output format is RAW, ignoring.");
						palette_pathbuf.clear();
					}
					_ => (),
				}
			},
			
			_ => {
				println!("param_validator::validate() error: Errored while attempting to locate palette, ignoring");
				palette_pathbuf.clear();
			}
		}
	}
	
	// Return neatly packed list of arguments.
	let final_source: PathBuf;
	let final_directory_mode: bool;
	
	// Set final source depending on whether or not it's a directory,
	// returning None if an error is encountered
	if source_file_stem == "*" {
		match source_base_path.try_exists() {
			Ok(true) => {
				final_directory_mode = true;
				final_source = source_base_path;
			},
			
			Ok(false) => {
				println!("Could not locate source path, aborting operation.");
				return None;
			},
			
			_ => {
				println!("param_validator::validate() error: Errored while attempting to locate source path");
				return None;
			},
		}
	}
	
	else {
		match source_pathbuf.try_exists() {
			Ok(true) => {
				final_directory_mode = false;
				final_source = source_pathbuf;
			},
			
			Ok(false) => {
				println!("Could not locate source file, aborting operation.");
				return None;
			},
			
			_ => {
				println!("param_validator::validate() error: Errored while attempting to locate source file");
				return None;
			},
		}
	}

	// Final output
	return Some(Parameters {
		directory_mode: final_directory_mode,
		source_path: final_source,
		target_path: target_path,
		palette_file: palette_pathbuf,
		source_format: source_format,
		target_format: target_format,
		palette_transfer: palette_transfer,
		forced_bit_depth: forced_bit_depth,
		bit_depth: bit_depth,
		uncompressed: uncompressed,
		reindex: reindex,
		verbose: verbose,
		overwrite: overwrite,
	});
}