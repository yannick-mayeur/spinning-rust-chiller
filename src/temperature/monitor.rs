use anyhow::{Context, Result};
use log::{error, warn, debug, info};
use std::process::Command;

use crate::config::Config;

pub fn get_hdd_temp(device: &str) -> Result<i32> {
    debug!("Attempting to read temperature from {}", device);

    // Check if we have permission to read the device
    let output = Command::new("smartctl")
        .args(&["-a", device])
        .output()
        .with_context(|| format!("Failed to execute smartctl for {}. Try running with sudo", device))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "smartctl command failed for {}: {}",
            device,
            stderr
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    debug!("Raw smartctl output for {}: {}", device, stdout);

    // Look for different temperature attribute formats
    let temp = stdout
        .lines()
        .find(|line| {
            line.contains("Temperature_Celsius") || 
            line.contains("Current Drive Temperature") ||
            line.contains("Airflow_Temperature_Cel")
        })
        .and_then(|line| {
            debug!("Found temperature line: {}", line);
            if line.contains("Temperature_Celsius") {
                line.split_whitespace().nth(9)
            } else if line.contains("Current Drive Temperature") {
                line.split(':').nth(1).map(|s| s.trim().split_whitespace().next()).flatten()
            } else {
                line.split_whitespace().nth(9)
            }
        })
        .and_then(|temp| temp.parse().ok())
        .ok_or_else(|| {
            error!("Temperature data not found in smartctl output for {}", device);
            error!("Full smartctl output:\n{}", stdout);
            anyhow::anyhow!("Could not parse temperature for {}", device)
        })?;

    info!("Successfully read temperature {}°C from {}", temp, device);
    Ok(temp)
}

pub fn get_hottest_temp(config: &Config) -> Result<i32> {
    let mut max_temp = i32::MIN;
    let mut success = false;
    let mut errors = Vec::new();

    for hdd in &config.hdds {
        match get_hdd_temp(hdd) {
            Ok(temp) => {
                debug!("Temperature for {}: {}°C", hdd, temp);
                max_temp = max_temp.max(temp);
                success = true;
            }
            Err(e) => {
                errors.push(format!("{}: {}", hdd, e));
                warn!("Failed to read temperature from {}: {}", hdd, e);
            }
        }
    }

    if success {
        Ok(max_temp)
    } else {
        Err(anyhow::anyhow!(
            "Failed to read temperature from any HDD. Errors:\n{}",
            errors.join("\n")
        ))
    }
}
