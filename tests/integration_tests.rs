use spinning_rust_chiller::{
    config::Config,
    fan::calculate_pwm,
};

#[test]
fn test_pwm_calculation() {
    let config = Config::default();
    
    // Test minimum temperature
    assert_eq!(calculate_pwm(config.temp_low, &config), config.min_speed);
    
    // Test maximum temperature
    assert_eq!(calculate_pwm(config.temp_high, &config), config.max_speed);
    
    // Test middle temperature
    let mid_temp = (config.temp_low + config.temp_high) / 2;
    let mid_speed = calculate_pwm(mid_temp, &config);
    assert!(mid_speed > config.min_speed && mid_speed < config.max_speed);
}
