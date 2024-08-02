# ffmpeg file size optimalizer

Just a tool I made so I can send vids through communication apps with max quality while. Recently upgared it so you can pick your own codec and tolerance.

# Usage

#### Clone the repo and enter the direcotry:
```
git clone https://github.com/Nexenek/ffmpegfilesizeoptimalizer.git
cd ./ffmpegfilesizeoptimalizer
```

#### Build it:
```
cargo build --release
```

#### Runing the script:
The script takes 5 required arguments
- input_file - path to the input file
- output_file - path to the output file
- target_size_mb - targetted file size in Megabytes (script takes in floating point numbers)
- tolerance - what is the max differnece between output file size and targeted file size (once again script takes in floating point numbers)
- codec - you can define your own codec so you can optimize ffmpeg for your machine.

```
./target/release/ffmpegfilesizeoptimalizer input_file output_file target_size_mb tolerance codec
```