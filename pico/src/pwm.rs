use embassy_rp::pac::pwm;
use rp2040_pac::PWM;

use crate::math;

const DEFAULT_TOP: u16 = 0x8000;
const DEFAULT_BOT: u16 = 0x0000;
const NUM_PINS: usize = 30;

// TODO: Review the publicity of everything

mod pwm_enums {
    #[derive(Copy, Clone)]
    pub enum PwmSlice {
        A,
        B
    }
    
    #[derive(Copy, Clone)]
    pub enum PwmChannel {
        Channel0 = 0,
        Channel1 = 1,
        Channel2 = 2,
        Channel3 = 3,
        Channel4 = 4,
        Channel5 = 5,
        Channel6 = 6,
        Channel7 = 7
    }
    
    #[derive(Copy, Clone)]
    pub enum PwmStatus {
        Enabled,
        Disabled
    }
}

#[derive(Copy, Clone)]
pub struct PwmPin {
    id: u8,
    slice: pwm_enums::PwmSlice,
    channel: pwm_enums::PwmChannel,
    status: pwm_enums::PwmStatus,
}

pub struct PWMDriver {
    pwm: PWM,
    pins: [PwmPin; NUM_PINS]
}

impl PWMDriver {
    pub fn begin() -> Self {
        PWMDriver {
            pwm: unsafe {rp2040_pac::Peripherals::steal().PWM },
            pins: {
                let mut pins = [PwmPin {
                    id: 0,
                    slice: pwm_enums::PwmSlice::A,
                    channel: pwm_enums::PwmChannel::Channel0,
                    status: pwm_enums::PwmStatus::Disabled
                }; NUM_PINS];

                for i in 0..NUM_PINS {
                    if i % 2 != 0 {
                        pins[i].slice = pwm_enums::PwmSlice::B;
                    }
                    pins[i].id = i as u8;
                }

                // Set all the channels for the PWMs
                // This is the best way that I could thing of to do this :(
                pins[0].channel =   pwm_enums::PwmChannel::Channel0;
                pins[1].channel =   pwm_enums::PwmChannel::Channel0;
                pins[2].channel =   pwm_enums::PwmChannel::Channel1;
                pins[3].channel =   pwm_enums::PwmChannel::Channel1;
                pins[4].channel =   pwm_enums::PwmChannel::Channel2;
                pins[5].channel =   pwm_enums::PwmChannel::Channel2;
                pins[6].channel =   pwm_enums::PwmChannel::Channel3;
                pins[7].channel =   pwm_enums::PwmChannel::Channel3;
                pins[8].channel =   pwm_enums::PwmChannel::Channel4;
                pins[9].channel =   pwm_enums::PwmChannel::Channel4;
                pins[10].channel =  pwm_enums::PwmChannel::Channel5;
                pins[11].channel =  pwm_enums::PwmChannel::Channel5;
                pins[12].channel =  pwm_enums::PwmChannel::Channel6;
                pins[13].channel =  pwm_enums::PwmChannel::Channel6;
                pins[14].channel =  pwm_enums::PwmChannel::Channel7;
                pins[15].channel =  pwm_enums::PwmChannel::Channel7;

                pins[16].channel =  pwm_enums::PwmChannel::Channel0;
                pins[17].channel =  pwm_enums::PwmChannel::Channel0;
                pins[18].channel =  pwm_enums::PwmChannel::Channel1;
                pins[19].channel =  pwm_enums::PwmChannel::Channel1;
                pins[20].channel =  pwm_enums::PwmChannel::Channel2;
                pins[21].channel =  pwm_enums::PwmChannel::Channel2;
                pins[22].channel =  pwm_enums::PwmChannel::Channel3;
                pins[23].channel =  pwm_enums::PwmChannel::Channel3;
                pins[24].channel =  pwm_enums::PwmChannel::Channel4;
                pins[25].channel =  pwm_enums::PwmChannel::Channel4;
                pins[26].channel =  pwm_enums::PwmChannel::Channel5;
                pins[27].channel =  pwm_enums::PwmChannel::Channel5;
                pins[28].channel =  pwm_enums::PwmChannel::Channel6;
                pins[29].channel =  pwm_enums::PwmChannel::Channel6;

                pins
            }
        }
    }

    // TODO: make a way so that there is a group for each channel so if 1 pwm is disabled then it wont disable all of them
    // starts a pwm pin (make sure the enable the pwm setting on the GPIO pin)
    pub fn start_pwm(&self, pin: usize) {
        let channel = self.pins[pin].channel as usize;

        self.pwm.ch(channel).top().write(|w| unsafe {
            w.top().bits(DEFAULT_TOP)
        });

        match (self.pins[pin].slice) {
            pwm_enums::PwmSlice::A => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.a().bits(DEFAULT_BOT)
                });
            },
            pwm_enums::PwmSlice::B => {
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
    pub fn set_pwm_value(&self, pin: usize, value: u16) {
        let channel = self.pins[pin].channel as usize;
        let slice = self.pins[pin].slice;

        match (slice) {
            pwm_enums::PwmSlice::A => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.a().bits(value)
                });
            },
            pwm_enums::PwmSlice::B => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.b().bits(value)
                });
            },
        } 
    }

    // TODO: add error checking if percent is above 100
    pub fn set_pwm_value_percent(&self, pin: usize, percent: f32) {
        let channel = self.pins[pin].channel as usize;
        let slice = self.pins[pin].slice;

        let value = math::map32(percent as f32, 0.0, 1.0, DEFAULT_BOT as f32, DEFAULT_TOP as f32) as u16;

        match (slice) {
            pwm_enums::PwmSlice::A => {
                self.pwm.ch(channel).cc().write(|w| unsafe {
                    w.a().bits(value)
                });
            },
            pwm_enums::PwmSlice::B => {
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