use anyhow::Result;
use std::{fs::File, io::Write};

use crate::config::Config;

pub fn calculate_pwm(temp: i32, config: &Config) -> u8 {
    if temp <= config.temp_low {
        return config.min_speed;
    }
    if temp >= config.temp_high {
        return config.max_speed;
    }

    let temp_range = (config.temp_high - config.temp_low) as f32;
    let speed_range = (config.max_speed - config.min_speed) as f32;
    let temp_delta = (temp - config.temp_low) as f32;

    (config.min_speed as f32 + (speed_range * temp_delta / temp_range)) as u8
}

pub fn set_fan_speed(pwm: u8, config: &Config) -> Result<()> {
    File::create(&config.fan_control_path)?
        .write_all(pwm.to_string().as_bytes())?;
    Ok(())
}
