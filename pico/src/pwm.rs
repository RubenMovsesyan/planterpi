use rp2040_pac::PWM;

use crate::math;

const DEFAULT_TOP: u16 = 0x8000;
const DEFAULT_BOT: u16 = 0x0000;


pub enum PwmSlice {
    A,
    B
}

pub struct PWMDriver {
    pwm: PWM
}

impl PWMDriver {
    pub fn begin() -> Self {
        PWMDriver {
            pwm: unsafe {rp2040_pac::Peripherals::steal().PWM }
        }
    }

    // TODO: fix this to allow setting both PWM slices if one is already active
    pub fn start_pwm(&self, channel: usize, slice: PwmSlice) {
        self.pwm.ch(channel).top().write(|w| unsafe {
            w.top().bits(DEFAULT_TOP)
        });

        match (slice) {
            PwmSlice::A => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.a().bits(DEFAULT_BOT)
                });
            },
            PwmSlice::B => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.b().bits(DEFAULT_BOT)
                });
            },
        }

        self.pwm.ch(channel).csr().write(|w| {
            w.en().bit(true)
        });
    }

    // TODO: add error checking if value is above 0x8000
    pub fn set_pwm_value(&self, channel: usize, slice: PwmSlice, value: u16) {
        match (slice) {
            PwmSlice::A => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.a().bits(value)
                });
            },
            PwmSlice::B => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.b().bits(value)
                });
            },
        } 
    }

    // TODO: add error checking if percent is above 100
    pub fn set_pwm_value_percent(&self, channel: usize, slice: PwmSlice, percent: f32) {
        let value = math::map32(percent as f32, 0.0, 1.0, DEFAULT_BOT as f32, DEFAULT_TOP as f32) as u16;

        // log::info!("Percent: {}\n\rValue: {:#06x}", percent, value);

        match (slice) {
            PwmSlice::A => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.a().bits(value)
                });
            },
            PwmSlice::B => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.b().bits(value)
                });
            },
        }
    }

    // ! This one was for debugging purposes, make into better read function
    #[deprecated]
    pub fn read_info(&self, channel: usize) {
        log::info!("Top Register:       {:#010x}", self.pwm.ch(channel).top().read().bits());
        log::info!("Compare Register:   {:#010x}", self.pwm.ch(channel).cc().read().bits());
        log::info!("Control Register:   {:#010x}", self.pwm.ch(channel).csr().read().bits());
    }
}

// pub fn get_pwm_config() -> Config {
//     let mut config: Config = Default::default();
//     config.top = DEFAULT_TOP;
//     config.compare_b = DEFAULT_BOT;
//     config
// }

// // This is only for convenience if you have 3 channels for r g and b
// pub fn set_rgb(rgb: u32, rgb_config: (&mut Config, &mut Config, &mut Config)) {
//     // Separate out all variables for easy use
//     let red_config =        rgb_config.0;
//     let green_config =      rgb_config.1;
//     let blue_config =       rgb_config.2;

//     let rgb_converted =         math::color_math::u32_to_rgb(rgb);

//     // Set all the compare values
//     red_config.compare_b =      math::map32(rgb_converted.0 as i32, 0, 255, DEFAULT_TOP as i32, DEFAULT_BOT as i32) as u16;
//     green_config.compare_b =    math::map32(rgb_converted.1 as i32, 0, 255, DEFAULT_TOP as i32, DEFAULT_BOT as i32) as u16;
//     blue_config.compare_b =     math::map32(rgb_converted.2 as i32, 0, 255, DEFAULT_TOP as i32, DEFAULT_BOT as i32) as u16;
// }