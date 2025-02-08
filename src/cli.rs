use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "spinning-rust-chiller", about = "HDD temperature-based fan control")]
pub struct Cli {
    /// Path to config file (optional)
    #[structopt(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// Fan control PWM file path
    #[structopt(long, parse(from_os_str))]
    pub pwm_path: Option<PathBuf>,

    /// HDD devices to monitor (e.g., /dev/sda)
    #[structopt(long)]
    pub hdds: Vec<String>,

    /// Generate example config file
    #[structopt(long)]
    pub generate_config: Option<PathBuf>,
}
