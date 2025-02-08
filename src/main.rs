// src/main.rs
use anyhow::Result;
use log::{error, info};
use std::{process, thread, time::Duration};
use structopt::StructOpt;
use std::process::Command;
use log::debug;


use spinning_rust_chiller::{
    cli::Cli,
    config::Config,
    fan::{calculate_pwm, set_fan_speed},
    temperature::get_hottest_temp,
};

fn check_prerequisites() -> Result<()> {
    let smartctl = Command::new("smartctl")
        .arg("--version")
        .output()?;
    if !smartctl.status.success() {
        return Err(anyhow::anyhow!("smartctl not found or not working. Please install smartmontools"));
    }
    Ok(())
}

fn main() -> Result<()> {
   env_logger::init();
   
   if std::env::var("RUST_LOG").is_err() {
       std::env::set_var("RUST_LOG", "debug");
   }

   debug!("Working directory: {:?}", std::env::current_dir()?);
   debug!("Environment PATH: {:?}", std::env::var("PATH"));
   
   debug!("Checking prerequisites...");
   check_prerequisites()?;
   
   debug!("Parsing CLI args...");
   let cli = Cli::from_args();
   debug!("CLI args: {:?}", cli);  // Assuming you derive Debug for Cli

   if let Some(path) = cli.generate_config {
       debug!("Generating example config at {:?}", path);
       Config::save_example_config(&path)?;
       info!("Example config file generated at {:?}", path);
       process::exit(0);
   }

   debug!("Loading config...");
   let config = match Config::load(&cli) {
       Ok(config) => {
           debug!("Config loaded successfully: {:?}", config);
           config
       }
       Err(e) => {
           error!("Configuration error: {:#}", e);  // Use {:#} for detailed error
           process::exit(1);
       }
   };

   info!("Starting HDD temperature monitoring");
   info!("Monitoring HDDs: {:?}", config.hdds);
   info!("Using PWM control file: {}", config.fan_control_path);

   // Main loop
   loop {
       debug!("Starting temperature check cycle");
       match get_hottest_temp(&config) {
           Ok(temp) => {
               let pwm = calculate_pwm(temp, &config);
               info!("Hottest HDD Temp: {}°C -> Setting Fan Speed: {}", temp, pwm);
               
               if let Err(e) = set_fan_speed(pwm, &config) {
                   error!("Failed to set fan speed: {:#}", e);
               }
           }
           Err(e) => error!("Failed to get temperature: {:#}", e),
       }
       thread::sleep(Duration::from_secs(config.interval));
   }
}
