use cortex_m::peripheral;
use rp2040_pac::{SPI0, SPI1};

pub enum SPISelector {
    Spi0,
    Spi1
}

pub struct SPIDriver {
    spi0: SPI0,
    spi1: SPI1,
}

impl SPIDriver {
    pub fn begin() -> Self {
        // Set the peripheral clock speed of the rp2040
        // let clock = unsafe { rp2040_pac::Peripherals::steal().CLOCKS };

        // clock.clk_peri_ctrl().write(|w| {
        //     w.auxsrc().clk_sys();
        //     w.enable().set_bit()
        // });

        SPIDriver {
            spi0: unsafe { rp2040_pac::Peripherals::steal().SPI0 },
            spi1: unsafe { rp2040_pac::Peripherals::steal().SPI1 }
        }
    }

    // Peripheral frequency is usually 125MHz for the rp2040
    pub fn set_baud_rate(
        &self, 
        peripheral_frequency: u32,
        baudrate: u32,
        spi_selector: SPISelector
    ) -> (u8, u8) {
        let freq = peripheral_frequency;
        let baud = baudrate;

        let mut prescale: u32 = u8::MAX as u32;
        let mut postdiv: u32 = 0x00;

        // Taken directly from the rp2040 hal
        // Find the smallest prescale value which puts the frequency in the range for the post-div
        for prescale_option in (2u32..=254).step_by(2) {
            if freq < ((prescale_option + 2) * 256).saturating_mul(baud) {
                prescale = prescale_option;
                break;
            }
        }

        // Find the highest post-div that will make the output <= baudrate
        for postdiv_option in (0u32..=255).rev() {
            if freq / (prescale * postdiv_option) > baudrate {
                postdiv = postdiv_option;
                break;
            }
        }

        // Return values so we dont have to typecase more than once
        let pscl = prescale as u8;
        let posd = postdiv as u8;

        match (spi_selector) {
            SPISelector::Spi0 => {
                self.spi0.sspcpsr().write(|w| unsafe {
                    w.cpsdvsr().bits(pscl)
                });
                self.spi0.sspcr0().write(|w| unsafe {
                    w.scr().bits(posd)
                });
            },
            SPISelector::Spi1 => {
                self.spi1.sspcpsr().write(|w| unsafe {
                    w.cpsdvsr().bits(pscl)
                });
                self.spi1.sspcr0().write(|w| unsafe {
                    w.scr().bits(posd)
                });
            },
        }

        (pscl, posd)
    }

    // data size ranges from 0b0011 to 0b1111 (3 - 15)
    pub fn send_data<const COUNT: usize>(
        &self,
        data_size: u8,
        data: &[u16],
        spi_selector: SPISelector
    ) {
        match (spi_selector) {
            SPISelector::Spi0 => {
                // Set up the control registers
                self.spi0.sspcr0().write(|w| unsafe {
                    w.dss().bits(data_size)
                });

                // Set to master mode
                // Enable the port
                self.spi0.sspcr1().write(|w| unsafe {
                    w.ms().clear_bit();
                    w.sse().set_bit()
                });


                // Write the data to the FIFO
                for buffer_index in 0..COUNT {
                    self.spi0.sspdr().write(|w| unsafe {
                        w.data().bits(data[buffer_index])
                    });

                    // Wait for the transmit FIFO to be clear
                    while self.spi0.sspsr().read().tnf().bit_is_clear() {}
                }
            },
            SPISelector::Spi1 => {
                // Set up the control registers
                self.spi1.sspcr0().write(|w| unsafe {
                    w.dss().bits(data_size)
                });

                // Set to master mode
                // Enable the port
                self.spi1.sspcr1().write(|w| unsafe {
                    w.ms().clear_bit();
                    w.sse().set_bit()
                });

                // Write the data to the FIFO
                for buffer_index in 0..COUNT {
                    self.spi1.sspdr().write(|w| unsafe {
                        w.data().bits(data[buffer_index])
                    });

                    // Wait for the transmit FIFO to be clear
                    while self.spi1.sspsr().read().tnf().bit_is_clear() {}
                }
            },
        }
    }

    pub fn read_registers(&self, spi_selector: SPISelector) {
        match (spi_selector) {
            SPISelector::Spi0 => {
                log::info!("SPI0");
                log::info!("    SSPCR0  Register: {:#010x}", self.spi0.sspcr0().read().bits());
                log::info!("    SSPCR1  Register: {:#010x}", self.spi0.sspcr1().read().bits());
                log::info!("    SSPDR   Register: {:#010x}", self.spi0.sspdr().read().bits());
                log::info!("    SSPCPSR Register: {:#010x}", self.spi0.sspcpsr().read().bits());
            },
            SPISelector::Spi1 => {
                log::info!("SPI1");
                log::info!("    SSPCR0  Register: {:#010x}", self.spi1.sspcr0().read().bits());
                log::info!("    SSPCR1  Register: {:#010x}", self.spi1.sspcr1().read().bits());
                log::info!("    SSPDR   Register: {:#010x}", self.spi1.sspdr().read().bits());
                log::info!("    SSPCPSR Register: {:#010x}", self.spi1.sspcpsr().read().bits());
            },
        }
    }
}