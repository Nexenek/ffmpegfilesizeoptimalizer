use std::process::Command;
use structopt::StructOpt;
use std::fs;
use std::str;

#[derive(StructOpt)]
struct Cli {
    #[structopt(help = "Path to the input file")]
    input_file: String,
    #[structopt(help = "Path to the output file")]
    output_file: String,
    #[structopt(help = "Target file size in Megabytes (accepts floating-point numbers)", parse(try_from_str = parse_f64))]
    target_size_mb: f64,
    #[structopt(help = "Maximum allowed difference between the output file size and the target file size (accepts floating-point numbers)", parse(try_from_str = parse_f64))]
    tolerance: f64,
    #[structopt(help = "Define your own codec to optimize `ffmpeg` for your machine. Defaults to `libx264`", default_value = "libx264")]
    codec: String,
    #[structopt(help = "Hardware acceleration to use. Defaults to `auto`", default_value = "auto")]
    hwaccel: String,
}

fn parse_f64(src: &str) -> Result<f64, std::num::ParseFloatError> {
    src.parse::<f64>()
}

fn get_video_duration(input_file: &str) -> f64 {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error", "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1", input_file
        ])
        .output()
        .expect("Failed to execute ffprobe");

    let duration_str = str::from_utf8(&output.stdout).expect("Invalid UTF-8 sequence");
    duration_str.trim().parse::<f64>().expect("Failed to parse duration")
}

fn calculate_bitrate(target_size_mb: f64, duration_seconds: f64) -> f64 {
    let target_size_bits = target_size_mb * 8.0 * 1024.0 * 1024.0;
    target_size_bits / duration_seconds
}

fn get_file_size(file_path: &str) -> f64 {
    let metadata = fs::metadata(file_path).expect("Unable to read file metadata");
    metadata.len() as f64 / (1024.0 * 1024.0)
}

fn compress_video(input_file: &str, output_file: &str, target_size_mb: f64, tolerance: f64, codec: &str, hwaccel: &str) {
    let duration = get_video_duration(input_file);
    let max_iterations = 10;
    let tolerance =  tolerance;

    for iteration in 0..max_iterations {
        let bitrate = calculate_bitrate(target_size_mb, duration);
        let bitrate_kbps = (bitrate / 1000.0) as i32;

        let pass1_output = Command::new("ffmpeg")
            .args(&[
                "-y", "-hwaccel", hwaccel, "-c:v", codec, "-i", input_file, "-b:v", &format!("{}k", bitrate_kbps),
                "-c:v", codec, "-pass", "1", "-f", "mp4", "NUL"
            ])
            .output()
            .expect("Failed to execute ffmpeg pass 1");

        if !pass1_output.status.success() {
            eprintln!(
                "FFmpeg pass 1 failed: {}",
                str::from_utf8(&pass1_output.stderr).unwrap()
            );
            return;
        }

        let pass2_output = Command::new("ffmpeg")
            .args(&[
                "-y", "-hwaccel", hwaccel, "-c:v", codec, "-i", input_file, "-b:v", &format!("{}k", bitrate_kbps),
                "-c:v", codec, "-pass", "2", output_file
            ])
            .output()
            .expect("Failed to execute ffmpeg pass 2");

        if !pass2_output.status.success() {
            eprintln!(
                "FFmpeg pass 2 failed: {}",
                str::from_utf8(&pass2_output.stderr).unwrap()
            );
            return;
        }

        let current_size = get_file_size(output_file);
        println!("Iteration: {}, Current Size: {:.2} MB, Target Size: {:.2} MB",
                 iteration + 1, current_size, target_size_mb);

        if (current_size - target_size_mb).abs() <= tolerance {
            break;
        }
    }
}

fn main() {
    let args = Cli::from_args();

    if !fs::metadata(&args.input_file).is_ok() {
        eprintln!("Input file does not exist or is not accessible");
        return;
    }

    compress_video(&args.input_file, &args.output_file, args.target_size_mb, args.tolerance, &args.codec, &args.hwaccel);
    println!("Compressed video saved as {}", args.output_file);
}