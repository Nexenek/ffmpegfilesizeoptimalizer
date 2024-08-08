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
    println!("Begginging analysis...");
    let duration = get_video_duration(input_file);
    println!("Duration: {}", duration);
    let mut bitrate = calculate_bitrate(target_size_mb, duration);
    println!("Calculated Bitrate: {}", bitrate);
    let max_iterations = 10;

    println!("Beggining compression...");
    for iteration in 0..max_iterations {
        let bitrate_kbps = (bitrate / 1000.0) as i32;
        println!("Calculated Bitrate (kbps): {}", bitrate_kbps);

        let pass1_output = Command::new("ffmpeg")
            .args(&[
                "-y", "-hwaccel", hwaccel, "-i", input_file, "-b:v", &format!("{}k", bitrate_kbps),
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
                "-y", "-hwaccel", hwaccel, "-i", input_file, "-b:v", &format!("{}k", bitrate_kbps),
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

        if current_size > target_size_mb {
            if current_size - target_size_mb >= 30.0 {
                bitrate *= 0.80;
                println!("Reducing bitrate by 20%");
            } else if current_size - target_size_mb >= 8.5 { 
                bitrate *= 0.85;
                println!("Reducing bitrate by 15%");
            } else {
                bitrate *= 0.9;
                println!("Reducing bitrate by 10%");
            }
        } else {
            if target_size_mb - current_size >= 30.0 {
                bitrate *= 1.20;
                println!("Increasing bitrate by 20%");
            } else if target_size_mb - current_size >= 8.5 {
                bitrate *= 1.15;
                println!("Increasing bitrate by 15%");
            } else {
                bitrate *= 1.1;
                println!("Increasing bitrate by 10%");
            }
        }
    }
}

fn main() {
    let args = Cli::from_args();
    println!("Input file: {}", args.input_file);
    println!("Output file: {}", args.output_file);
    println!("Target size: {:.2} MB", args.target_size_mb);
    println!("Tolerance: {:.2} MB", args.tolerance);
    println!("Codec: {}", args.codec);
    println!("Hardware acceleration: {}", args.hwaccel);

    if !fs::metadata(&args.input_file).is_ok() {
        eprintln!("Input file does not exist or is not accessible");
        return;
    }

    compress_video(&args.input_file, &args.output_file, args.target_size_mb, args.tolerance, &args.codec, &args.hwaccel);
    println!("Compressed video saved as {}", args.output_file);
}
