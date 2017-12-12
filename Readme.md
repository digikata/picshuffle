
# picshuffle
A utility to grab piles of photo files and organize them into a destination directory

Photos are fingerprinted by hash and unique files are sent to the output directory
further split up by year/month subdirectories.


## Changelog

v0.1.3
Added a fast hash option to only hash the first n bytes of each file

v0.1.2
Identical output file names (with different contents) are deconflicted by automatic rename


## Todo
* Use EXIF file creation dates when available
* Generate dry run output which could be saved as a shell script as an alternate way of
  execution
* Play with parallelizing the scan
At least the hash, file pairs..
* add exclude syntax for scan
Tap into ignore crate walk build options from command line args

