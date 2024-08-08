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
 `-input <file>` or `-i <file>`
 Specifies the input file or files. In order to process entire directories, use `*` as the filename.<br/>
 For example: `-input <path>/*.png`. The input format is still required (`*.png`, `*.raw`, `*.bin`, `*.bmp`).

 `-format <format>` or `-f <format>`
 Specifies the output format. RAW output appends PalMod naming automatically. PNG output is grayscale by default. BIN output is compressed by default.

 `-output <path>` or `-o <path>`
 Specifies the output path. Will be created if it doesn't exist. Defaults to the current working directory if not specified.

 `-palette <file>` or `-p <file>`
 Specifies the input palette. Works only with PNGs and BMPs. Will accept any file, but will only produce expected results with ACT-format palettes.

 `-palcopy` or `-c`
 Copies the source sprite's palette to the output sprite. Takes precedence over `-palette`.

 `-reindex` or `-r`
 Reindexes the output sprite from 1-2-3-4 to 1-3-2-4 and vice versa.

 `-uncompressed` or `-u`
 Outputs uncompressed sprites if the output format is BIN.

 `-overwrite` or `-w`
 Enables overwriting pre-existing files. Can overwrite files in place.

 `-list` or `-l`
 Prints each file name to the console as it processes sprites.

## Usage Examples
 - `ghoul -input *.png -format bin -output destination`
 Processes every PNG file in the current directory, and outputs compressed BIN sprites to a directory called `destination`.

 - `ghoul -input source/*.bmp -reindex -output source -overwrite`
 Overwrites every BMP file at the `source` directory with reindexed versions.

 - `ghoul -input sprite_0.bin -palette pal.act -f png`
 Creates an indexed PNG file called `sprite_0.png` with `pal.act` as its palette.