mod validation;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

use crate::cli::Cli;
use validation::validate_config;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub hdds: Vec<String>,
    pub max_speed: u8,
    pub min_speed: u8,
    pub temp_low: i32,
    pub temp_high: i32,
    pub interval: u64,
    pub fan_control_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hdds: vec!["/dev/sda".to_string()],
            max_speed: 255,
            min_speed: 30,
            temp_low: 35,
            temp_high: 50,
            interval: 10,
            fan_control_path: "/sys/class/hwmon/hwmon0/pwm1".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let cli = Cli::from_args();
        let mut config = if let Some(ref config_path) = cli.config {
            Self::from_file(config_path)?
        } else {
            Config::default()
        };

        // Override with CLI arguments
        if !cli.hdds.is_empty() {
            config.hdds = cli.hdds;
        }
        if let Some(pwm_path) = cli.pwm_path {
            config.fan_control_path = pwm_path.to_string_lossy().to_string();
        }

        validate_config(&config)?;
        Ok(config)
    }

    fn from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save_example_config(path: &PathBuf) -> Result<()> {
        let config = Config::default();
        let toml = toml::to_string_pretty(&config)?;
        fs::write(path, toml)?;
        Ok(())
    }
}
