use rp2040_pac::generic;
use rp2040_pac::generic::R;
use rp2040_pac::generic::Reg;
use rp2040_pac::io_bank0;
use rp2040_pac::io_bank0::gpio::gpio_ctrl::GPIO_CTRL_SPEC;
use rp2040_pac::io_bank0::gpio::gpio_status;
use rp2040_pac::io_bank0::gpio::gpio_ctrl;
use rp2040_pac::io_bank0::gpio::gpio_status::GPIO_STATUS_SPEC;
use rp2040_pac::io_bank0::gpio::GPIO_CTRL;
use rp2040_pac::IO_BANK0;

const NUM_GPIO: usize = 30;

pub enum CtrlStatus {
    Low,
    High,
    Pwm,
}

pub struct GPIODriver {
    io_bank0: IO_BANK0
}

impl GPIODriver {
    pub fn begin() -> Self {
        GPIODriver {
            io_bank0: unsafe { rp2040_pac::Peripherals::steal().IO_BANK0 }
        }
    }

    #[deprecated]
    pub fn enable_output(&self, pin: usize) {
        self.io_bank0.gpio(pin).gpio_ctrl().write(|w| {
            w.oeover().enable()
        });
    }

    pub fn set_pin(&self, pin: usize, status: CtrlStatus) {
        match (status) {
            CtrlStatus::Low => {
                self.io_bank0.gpio(pin).gpio_ctrl().write(|w| {
                    w.funcsel().sio();
                    w.oeover().enable();
                    w.outover().low()
                });
            },
            CtrlStatus::High => {
                self.io_bank0.gpio(pin).gpio_ctrl().write(|w| {
                    w.funcsel().sio();
                    w.oeover().enable();
                    w.outover().high()
                });
            }
            CtrlStatus::Pwm => {
                self.io_bank0.gpio(pin).gpio_ctrl().write(|w| {
                    w.funcsel().pwm()
                })
            },
        }
    }

    pub fn read_pin(&self, pin: usize) -> R<GPIO_CTRL_SPEC> {
        self.io_bank0.gpio(pin).gpio_ctrl().read()
    }
}