
# picshuffle
A utility to grab piles of photo files and organize them into a destination directory

## Features

## Todo

* Resolve identical file names w/ different hashes (by renaming the file...)
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

