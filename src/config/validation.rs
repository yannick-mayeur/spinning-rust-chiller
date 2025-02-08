use anyhow::Result;
use std::path::Path;

use super::Config;

pub fn validate_config(config: &Config) -> Result<()> {
    validate_hdds(&config.hdds)?;
    validate_pwm_file(&config.fan_control_path)?;
    validate_temperature_ranges(config.temp_low, config.temp_high)?;
    Ok(())
}

fn validate_hdds(hdds: &[String]) -> Result<()> {
    if hdds.is_empty() {
        return Err(anyhow::anyhow!("No HDDs specified to monitor"));
    }

    for hdd in hdds {
        if !Path::new(hdd).exists() {
            return Err(anyhow::anyhow!("HDD device not found: {}", hdd));
        }
    }
    Ok(())
}

fn validate_pwm_file(path: &str) -> Result<()> {
    let pwm_path = Path::new(path);
    if !pwm_path.exists() {
        return Err(anyhow::anyhow!("Fan control PWM file not found: {}", path));
    }
    Ok(())
}

fn validate_temperature_ranges(low: i32, high: i32) -> Result<()> {
    if low >= high {
        return Err(anyhow::anyhow!(
            "Low temperature threshold must be less than high temperature threshold"
        ));
    }
    Ok(())
}
