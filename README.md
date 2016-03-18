# rust-alsb
Rust implementation of an Advanced Least Significant Bit Steganography process.

Currently only hides data in image files. Will produce PNG files.

**Please note** this software is for educational purposes. It is probably not secure as better algorithms are available.

## Least Significant Bit Steganography
Hide bits in the least significant bits of a lossy file.

## What does Advanced Least Significant Bit Steganography add ?
Each file is complemented with a random buffer for extra security when dealing with multiple copies.

## What is the value of ALSB ?
With standard the standard LSB algorithm, if someone were to hide the same file in two different files, shared bits would be detectable.

Multiple copies of the content can be hidden without recurrent patterns in encoded media as random data removes shared LSB between copies.


## TODO
* Implement the ALSB part (currently only LSB)
* Test if ALSB is detectable (is the random part recurring?)
