# ghoul
 A command-line sprite manipulation tool for Guilty Gear XX AC+R

## Features
 - Can convert sprites between PNG, RAW, BIN, and BMP formats.
 - Can take in RAW sprites that use PalMod preview naming (name-W-width-H-height.raw)
 - Can take in grayscale, indexed, and RGB (using the red channel) PNGs
 - Can output both compressed (default) and uncompressed BINs
 - Can apply an external ACT palette to both PNGs and BMPs
 - Can reindex sprites in all four formats
 - Can operate on either a single sprite or entire directories at once
 - Can output to a specified directory, creating it if it doesn't exist
 - Will not overwrite pre-existing files unless told to

## Available Parameters
 - `-input <file>` or `-i <file>`<br/>
 Specifies the input file or files. In order to process entire directories, use `*` as the filename.<br/>
 For example: `-input <path>/*.png`. The input format is still required (`*.png`, `*.raw`, `*.bin`, `*.bmp`).

 - `-format <format>` or `-f <format>`<br/>
 Specifies the output format. RAW output appends PalMod naming automatically. PNG and BMP outputs are indexed. BIN output is compressed by default.

 - `-output <path>` or `-o <path>`<br/>
 Specifies the output path. Will be created if it doesn't exist. Defaults to the current working directory if not specified.

 - `-palette <file>` or `-p <file>`<br/>
 Specifies the input palette. Will accept any file, but will only produce expected results with ACT-format palettes. Doesn't work on RAWs.

 - `-palcopy` or `-c`<br/>
 Copies the source sprite's palette to the output sprite. Takes precedence over `-palette`. Doesn't work on RAWs.

 - `-force-4bpp` or `-4`<br/>
 Forces the output sprite to 4-bit color depth. Could produce incorrect results ingame if converting from an 8 bpp source.
 
 - `-force-8bpp` or `-8`<br/>
 Forces the output sprite to 8-bit color depth. Could produce incorrect results ingame if converting from a 4 bpp source.
 
 - `-as-rgb` or `-rgb`<br/>
 Forces input sprites to be treated as RGB, even if indexed. No effect on sprites without a palette.

 - `-reindex` or `-r`<br/>
 Reindexes the output sprite from 1-2-3-4 to 1-3-2-4 and vice versa.

 - `-uncompressed` or `-u`<br/>
 Outputs uncompressed sprites if the output format is BIN.

 - `-overwrite` or `-w`<br/>
 Enables overwriting pre-existing files. Can overwrite files in place.

 - `-list` or `-l`<br/>
 Prints each file name to the console as it processes sprites.

## Usage Examples
 - `ghoul -input *.png -format bin -output destination`<br/>
 Processes every PNG file in the current directory, and outputs compressed BIN sprites to a directory called `destination`.

 - `ghoul -input source/*.bmp -reindex -output source -overwrite`<br/>
 Overwrites every BMP file at the `source` directory with reindexed versions.

 - `ghoul -input sprite_0.bin -palette pal.act -format png`<br/>
 Creates a PNG file called `sprite_0.png` with `pal.act` as its palette.
 
 - `ghoul -input source/*.bin -palcopy -format png`
 Process every BIN file at the `source` directory, and outputs PNGs at the current directory. Will include palettes if present in the BIN files.