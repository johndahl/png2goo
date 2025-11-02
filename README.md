# Introduction
The purpose of this software is to generate a single layer goo file from a png file. The intended use case is to exposure uv sensitive resist films on PCBs, using a MSLA resin printer. 

Successful tests has been made on a Elegoo Saturn 4 Ultra. The code has only been tested on ubuntu 24.04, but should compile on other OS as well. 

# Install rust compiler
 curl https://sh.rustup.rs -sSf | sh

# Compile the code
cargo build --release

# Example usage
target/release/png2goo INPUT.png OUTPUT.goo 11520 5120 30 0.05

# Mandatory args 
- png input image
- output goo file
- width in pixels
- height in pixels
- exposure time in seconds
- layer height in mm


