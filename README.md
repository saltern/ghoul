# ghoul
 A command-line sprite manipulation tool for Guilty Gear XX AC+R

## Features
 - Can convert sprites between PNG, RAW, BIN, and BMP formats.
 - Can take in RAW sprites that use PalMod preview naming (name-W-width-H-height.raw)
 - Can take in grayscale, indexed, and RGB (using the red channel) PNGs
 - Can output both compressed (default) and uncompressed BINs
 - Can apply an external ACT palette to PNGs, BMPs, and BINs
 - Can reindex sprites in all four formats
 - Can operate on either a single sprite or entire directories at once
 - Can output to a specified directory, creating it if it doesn't exist
 - Will not overwrite pre-existing files unless told to

## Available Parameters
### File Parameters
 - `-input <file>` or `-i <file>`<br/>
 Specifies the input file or files. In order to process entire directories, use `*` as the filename.<br/>
 For example: `-input <path>/*.png`. The input format is still required (`*.png`, `*.raw`, `*.bin`, `*.bmp`).

 - `-output <path>` or `-o <path>`<br/>
 Specifies the output path. Will be created if it doesn't exist. Defaults to the current working directory if not specified.

 - `-format <format>` or `-f <format>`<br/>
 Specifies the output format. RAW output appends PalMod naming automatically. PNG and BMP outputs are indexed. BIN output is compressed by default.

 - `-overwrite` or `-w`<br/>
 Enables overwriting pre-existing files. Can overwrite files in place.

 - `-list` or `-l`<br/>
 Prints each file name to the console as it processes sprites.

### Palette Parameters
 - `-palette <file>` or `-p <file>`<br/>
 Specifies the input palette. Will accept any file, but will only produce expected results with ACT-format palettes.<br/>Doesn't work on RAWs.

 - `-palcopy` or `-c`<br/>
 Copies the source sprite's palette to the output sprite. Takes precedence over `-palette`.<br/>Doesn't work on RAWs.

 - `-opaque` or `-q`<br/>
 Makes every color in the input palette (when using `-palette` or `-palcopy`) completely opaque (sets alpha to 255).<br/>Doesn't work on RAWs.


### Image Processing Parameters
 - `-as-rgb` or `-rgb`<br/>
 Forces input sprites to be treated as RGB, even if indexed.<br/>No effect on sprites without a palette.

 - `-force-4bpp` or `-4`<br/>
 Forces the output sprite to 4-bit color depth. Could produce incorrect results ingame if converting from an 8 bpp source.
 
 - `-force-8bpp` or `-8`<br/>
 Forces the output sprite to 8-bit color depth. Could produce incorrect results ingame if converting from a 4 bpp source.

 - `-reindex` or `-r`<br/>
 Reindexes the output sprite from 1-2-3-4 to 1-3-2-4 and vice versa.

### BIN-Only Parameters
 - `-hash-set <number>` or `-hs <number>`<br/>
 Forces the hash of every output sprite to the specified `<number>` between 0 and 65535.

 - `-hash-inc <number>` or `-hi <number>`<br/>
 Writes unique, incremental hashes for every output sprite, starting at the specified `<number>` between 0 and 65535.

 - `-uncompressed` or `-u`<br/>
 Outputs uncompressed sprites.

## Usage Examples
 - `ghoul -input *.png -format bin -output destination`<br/>
 Converts all PNGs in the current directory to compressed BINs, saving the results to a directory called `destination`.<br/>This directory will be created if it doesn't already exist.

 - `ghoul -input source/*.bmp -reindex -output source -overwrite`<br/>
 Reindexes all BMPs in the `source` directory, overwriting the originals.

 - `ghoul -input sprite_0.bin -palette pal.act -format png`<br/>
 Converts `sprite_0.bin` to a PNG called `sprite_0.png`, coloring it with the palette contained in `pal.act`.<br/>In this example, all files are searched for in the current directory.
 
 - `ghoul -input source/*.bin -palcopy -format png`<br/>
 Converts all BINs in the `source` directory to PNGs, saving the results to the current directory.<br/>If the  BIN files contain palettes, they will be copied to the PNG results.
 
 - `ghoul -input *.png -palcopy -opaque -overwrite`<br/>
 Overwrites every PNG file in the current directory with completely opaque versions, if they have palettes.

 - `ghoul -input *.bin -output target -hash-inc 10`<br/>
 Gives every BIN file in the current folder a hash starting at 10 and incrementing per file, saving the results to a directory called `target`.