// src/main.rs
use anyhow::Result;
use log::{error, info};
use std::{process, thread, time::Duration};
use structopt::StructOpt;
use std::process::Command;
use log::warn;


use spinning_rust_chiller::{
    cli::Cli,
    config::Config,
    fan::{calculate_pwm, set_fan_speed},
    temperature::get_hottest_temp,
};

fn check_prerequisites() -> Result<()> {
    // Check if running as root/sudo
    let uid = unsafe { libc::geteuid() };
    if uid != 0 {
        warn!("Not running as root. You may need sudo privileges to read some devices");
    }

    // Check for smartctl
    let smartctl = Command::new("which")
        .arg("smartctl")
        .output()?;
    if !smartctl.status.success() {
        return Err(anyhow::anyhow!("smartctl not found. Please install smartmontools"));
    }

    Ok(())
}

fn main() -> Result<()> {
    // First: initialize logging
    env_logger::init();
    
    // Second: Set more verbose logging if needed
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    // Third: Check prerequisites right after logging is set up
    check_prerequisites()?;

    // Fourth: Parse CLI and handle config generation
    let cli = Cli::from_args();
    if let Some(path) = cli.generate_config {
        Config::save_example_config(&path)?;
        info!("Example config file generated at {:?}", path);
        process::exit(0);
    }

    // Fifth: Load the configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Configuration error: {}", e);
            process::exit(1);
        }
    };

    info!("Starting HDD temperature monitoring");
    info!("Monitoring HDDs: {:?}", config.hdds);
    info!("Using PWM control file: {}", config.fan_control_path);

    // Main loop
    loop {
        match get_hottest_temp(&config) {
            Ok(temp) => {
                let pwm = calculate_pwm(temp, &config);
                info!("Hottest HDD Temp: {}°C -> Setting Fan Speed: {}", temp, pwm);
                
                if let Err(e) = set_fan_speed(pwm, &config) {
                    error!("Failed to set fan speed: {}", e);
                }
            }
            Err(e) => error!("Failed to get temperature: {}", e),
        }

        thread::sleep(Duration::from_secs(config.interval));
    }
}
