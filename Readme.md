
# picshuffle
A utility to grab piles of photo files and organize them into a destination directory

## Changelog

v0.1.2
Identical output file names (with different contents) are deconflicted by automatic rename

## Todo

* Quick hash
hash first xx bytes
* Deconflict hash collisions
add file length to content hash? 
compare files on hash collisions (should be very infrequent anyway)
* EXIF file creation dates 
* Play with parallelizing the scan
At least the hash, file pairs..
* add exclude syntax for scan
Tap into ignore crate walk build options from command line args

