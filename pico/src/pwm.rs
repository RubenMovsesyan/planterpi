use embassy_rp::pwm::Config;

use crate::math;

const DEFAULT_TOP: u16 = 0x8000;
const DEFAULT_BOT: u16 = 8;

pub fn get_pwm_config() -> Config {
    let mut config: Config = Default::default();
    config.top = DEFAULT_TOP;
    config.compare_b = DEFAULT_BOT;
    config
}

// This is only for convenience if you have 3 channels for r g and b
pub fn set_rgb(rgb: u32, rgb_config: (&mut Config, &mut Config, &mut Config)) {
    // Separate out all variables for easy use
    let red_config =        rgb_config.0;
    let green_config =      rgb_config.1;
    let blue_config =       rgb_config.2;

    let rgb_converted =         math::color_math::u32_to_rgb(rgb);

    // Set all the compare values
    red_config.compare_b =      math::map32(rgb_converted.0 as i32, 0, 255, DEFAULT_TOP as i32, DEFAULT_BOT as i32) as u16;
    green_config.compare_b =    math::map32(rgb_converted.1 as i32, 0, 255, DEFAULT_TOP as i32, DEFAULT_BOT as i32) as u16;
    blue_config.compare_b =     math::map32(rgb_converted.2 as i32, 0, 255, DEFAULT_TOP as i32, DEFAULT_BOT as i32) as u16;
}