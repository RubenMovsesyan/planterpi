#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![allow(unused)] // Take this line out at the end of development

// Core string manipulation
use core::str;

// PAC Access
use gpio::CtrlStatus::{High, Low};


// Core libraries
use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_rp::{peripherals::{PIN_23, PIN_25}, spi::Polarity};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Pwm libraries
use embassy_rp::pwm::{Config, Pwm};

// SPI libraries
use embassy_rp::spi::Spi;
// use embassy_rp::spi;

// Wifi libraries
// use cyw43_pio::PioSpi;
// use embassy_net::Stack;
// use embassy_rp::peripherals::{DMA_CH0, PIO0};
// use embassy_rp::pio::Pio;

// Serial communication libraries
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use static_cell::StaticCell;

// Custom modules
mod ws2812b;
mod math;
mod pwm;
mod gpio;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

// #[embassy_executor::task]
// async fn wifi_task(runner: cyw43::Runner<'static, Output<'static, PIN_23>, PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>>) -> ! {
//     runner.run().await
// }

// #[embassy_executor::task]
// async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
//     stack.run().await
// }

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

// async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
//     let mut buf = [0; 64];
//     loop {
//         let n = class.read_packet(&mut buf).await?;
//         let data = &buf[..n];
//         info!("data: {:x}", data);
//         class.write_packet(data).await?;
//     }
// }

#[cortex_m_rt::pre_init]
unsafe fn before_main() {
    // Soft-reset doesn't clear spinlocks. Clear the one used by critical-section
    // before we hit main to avoid deadlocks when using a debugger
    embassy_rp::pac::SIO.spinlock(31).write_value(1);
}
// ----------------------------------- End of boilerplate --------------------------------------------------------

const DEFAULT_DELAY: u64 = 250;
const RED_LED: usize = 4;
const GREEN_LED: usize = 5;
const BLUE_LED: usize = 6;


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    /*
        Setup Section Begin (run once)
     */
    // JK there is more boilerplate :)
    let p = embassy_rp::init(Default::default());

    // Include wifi driver
    let fw = include_bytes!("../firmware/43439A0.bin");
    let clm = include_bytes!("../firmware/43439A0_clm.bin");

    // Set up usb serial
    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Required for windows compatibility. (windows is dumb >:( )
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;
        config
    };

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    // Create classes on the builder.
    // let mut class = {
    //     static STATE: StaticCell<State> = StaticCell::new();
    //     let state = STATE.init(State::new());
    //     CdcAcmClass::new(&mut builder, state, 64)
    // };


    // Create a class for the logger
    let logger_class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let logger_state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, logger_state, 64)
    };

    // Creates the logger and returns the logger future
    // Note: You'll need to use log::info! afterwards instead of info! for this to work (this also applies to all the other log::* macros)
    let log_fut = embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, logger_class);

    log::info!("Starting Serial");

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();
    // Serial setup complete

    // Setup wifi
    // let pwr = Output::new(p.PIN_23, Level::Low);
    // let cs = Output::new(p.PIN_25, Level::High);
    // let mut pio = Pio::new(p.PIO0);
    // let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p.PIN_24, p.PIN_29, p.DMA_CH0);
    // Now the boilerplate is over :)

    // Set up the pins for on and off configuration
    // let mut red_led = Output::new(p.PIN_1, Level::Low);  
    // let mut green_led = Output::new(p.PIN_2, Level::Low);    
    // let mut blue_led = Output::new(p.PIN_3, Level::Low);

    // Set up pins for PWM
    let mut red_config =    pwm::get_pwm_config();
    let mut green_config =  pwm::get_pwm_config();
    let mut blue_config =   pwm::get_pwm_config();

    let mut pwm_red =   Pwm::new_output_b(p.PWM_CH6, p.PIN_13, red_config.clone());
    let mut pwm_green = Pwm::new_output_b(p.PWM_CH7, p.PIN_15, green_config.clone());
    let mut pwm_blue =  Pwm::new_output_b(p.PWM_CH0, p.PIN_17, blue_config.clone());
    

    let mut set_rgb = |rgb: u32| {
        pwm::set_rgb(rgb, (&mut red_config, &mut green_config, &mut blue_config));
        pwm_red.set_config(&red_config);
        pwm_green.set_config(&green_config);
        pwm_blue.set_config(&blue_config);
    };


    // Addressable LED setup
    let spi0_mosi = p.PIN_3;
    let spi0_clk = p.PIN_2;

    // 300 us is the minimum for the reset period, the datasheets says 50us >:(
    let mut spi0 = Spi::new_blocking_txonly(p.SPI0, spi0_clk, spi0_mosi, ws2812b::get_addressable_led_config());

    let mut starting_hue = 0;
    let hue_adjust = 360 / ws2812b::NUM_LEDS;
    
    // let periph = unsafe { rp2040_pac::Peripherals::steal() };
    let gpio_pins = gpio::GPIODriver::begin();
    // gpio_pins.enable_output(4);
    /*
        Setup Section End
     */

    /*
        Loop Section Begin (run continuously)
     */
    // Putting the loop in an asynchronous function lets us run the loop and the logger at the same time
    let echo_fut = async {
        loop {
            gpio_pins.set_pin(RED_LED, Low);
            gpio_pins.set_pin(GREEN_LED, High);
            gpio_pins.set_pin(BLUE_LED, High);
            Timer::after_millis(500).await;
            gpio_pins.set_pin(RED_LED, High);
            gpio_pins.set_pin(GREEN_LED, Low);
            gpio_pins.set_pin(BLUE_LED, High);
            Timer::after_millis(500).await;
            gpio_pins.set_pin(RED_LED, High);
            gpio_pins.set_pin(GREEN_LED, High);
            gpio_pins.set_pin(BLUE_LED, Low);
            Timer::after_millis(500).await;
            log::info!("Cycle complete");


            // Timer::after_secs(2).await;
            // log::info!("Starting wait");
            // for i in (0..3).rev() {
            //     Timer::after_secs(1).await;
            //     log::info!("{}", i);
            // }

            // // This should set the gpio pin 4 to low
            // let periph = unsafe { rp2040_pac::Peripherals::steal() };
            // let io_bank0: rp2040_pac::IO_BANK0 = periph.IO_BANK0;
            // let gpio_ctrl: &rp2040_pac::generic::Reg<rp2040_pac::io_bank0::gpio::gpio_ctrl::GPIO_CTRL_SPEC> = io_bank0.gpio(4).gpio_ctrl(); // this is gpio pin 4
            // log::info!("GPIO4_CTRL pre: {:#06x}", gpio_ctrl.read().bits());
            // gpio_ctrl.write(|w| {
            //     w.oeover().enable(); // Enable the GPIO pin
            //     w.outover().low() // Set the pin to low
            // });
            // log::info!("GPIO4_CTRL post: {:#06x}", gpio_ctrl.read().bits());


            // let mut color_buffer: [u32; ws2812b::NUM_LEDS] = [0; ws2812b::NUM_LEDS];

            // for i in 0..ws2812b::NUM_LEDS {
            //     let color: f32 = ((starting_hue + (i * hue_adjust)) % 360) as f32;
            //     let rgb = math::color_math::hsl_to_rgb(color, 1.0, 0.5);
            //     // log::info!("Color: {:#06x}", rgb);
            //     color_buffer[i] = rgb;
            // }

            // let buf = ws2812b::generate_addressable_led_buffer::<{ ws2812b::LED_INFO_SIZE }>(&color_buffer);
            // set_rgb(color_buffer[0]);
            // spi0.blocking_write(&buf).unwrap();

            // if starting_hue == 0 {
            //     log::info!("Red: {:#06x}", color_buffer[0]);
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            // } else if starting_hue == 120 {
            //     log::info!("Green: {:#06x}", color_buffer[0]);
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            // } else if starting_hue == 240 {
            //     log::info!("Blue: {:#06x}", color_buffer[0]);
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(0);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            //     set_rgb(color_buffer[0]);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            // }

            // starting_hue += 1;
            // starting_hue %= 360;
            // Timer::after_millis(25).await;
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(echo_fut, log_fut)).await;

    /*
        Loop Section End
     */
}