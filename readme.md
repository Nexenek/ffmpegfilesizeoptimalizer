# FFmpeg File Size Optimizer

A tool designed to help you send videos through communication apps with maximum quality while meeting specific file size constraints. Recently upgraded to allow custom codec selection and tolerance levels.

## How It Works

The script performs initial calculations based on the video's bitrate. If the resulting file size doesn't fit within the specified tolerance, it iterates through further optimizations to ensure the highest quality while meeting your desired file size constraints.

## Usage

### Building the Tool

#### Clone the Repository and Enter the Directory
```bash
git clone https://github.com/Nexenek/ffmpegfilesizeoptimalizer.git
cd ffmpegfilesizeoptimalizer
```

#### Build It
```bash
cargo build --release
```

### Or Download from Releases

### Running the Script

The script requires `ffmpeg` and takes five required arguments:
- `input_file` - Path to the input file
- `output_file` - Path to the output file
- `target_size_mb` - Target file size in Megabytes (accepts floating-point numbers)
- `tolerance` - Maximum allowed difference between the output file size and the target file size (accepts floating-point numbers)
- `codec` - Define your own codec to optimize `ffmpeg` for your machine

```bash
./ffmpegfilesizeoptimalizer input_file output_file target_size_mb tolerance codec
```

---