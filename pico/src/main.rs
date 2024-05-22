#![no_std]
#![no_main]

use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_rp::{gpio, spi::Polarity};
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

// Pwm libraries
use embassy_rp::pwm::{Config, Pwm};

// SPI libraries
use embassy_rp::spi::Spi;
use embassy_rp::spi;

// Serial communication libraries
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            // EndpointError::BufferOverflow => panic!("Buffer overflow"),
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

const DEFAULT_DELAY: u64 = 10;
const DEFAULT_TOP: u16 = 0x8000;
const DEFAULT_BOT: u16 = 8;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    /*
        Setup Section Begin (run once)
     */
    // JK there is more boilerplate :)
    let p = embassy_rp::init(Default::default());

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
    // Now the boilerplate is over :)

    // Set up the pins for on and off configuration
    // let mut red_led = Output::new(p.PIN_1, Level::Low);  
    // let mut green_led = Output::new(p.PIN_2, Level::Low);    
    // let mut blue_led = Output::new(p.PIN_3, Level::Low);

    // Set up pins for PWM
    let mut pwm_config_red: Config = {
        let mut c: Config = Default::default();
        c.top = DEFAULT_TOP;
        c.compare_b = DEFAULT_BOT;
        c 
    };

    let mut pwm_config_green: Config = {
        let mut c: Config = Default::default();
        c.top = DEFAULT_TOP;
        c.compare_b = DEFAULT_BOT;
        c 
    };

    let mut pwm_config_blue: Config = {
        let mut c: Config = Default::default();
        c.top = DEFAULT_TOP;
        c.compare_b = DEFAULT_BOT;
        c 
    };

    // let mut pwm_red =   Pwm::new_output_b(p.PWM_CH0, p.PIN_1, pwm_config_red.clone());
    // let mut pwm_green = Pwm::new_output_b(p.PWM_CH1, p.PIN_3, pwm_config_green.clone());
    // let mut pwm_blue =  Pwm::new_output_b(p.PWM_CH2, p.PIN_5, pwm_config_blue.clone());

    // // Functions for setting the pwm pin values
    // let mut set_red = |compare_value: u16| {
    //     pwm_config_red.compare_b = compare_value;
    //     pwm_red.set_config(&pwm_config_red);
    // };

    // let mut set_green = |compare_value: u16| {
    //     pwm_config_green.compare_b = compare_value;
    //     pwm_green.set_config(&pwm_config_green);
    // };

    // let mut set_blue = |compare_value: u16| {
    //     pwm_config_blue.compare_b = compare_value;
    //     pwm_blue.set_config(&pwm_config_blue);
    // };


    // Addressable LED setup
    let spi1_mosi = p.PIN_3;
    let spi1_clk = p.PIN_2;

    let addressable_led_config = {
        let mut config = spi::Config::default();
        config.frequency = 8_500_000;
        config
    };

    const L0: u8 = 0xE0;
    const L1: u8 = 0xFC;

    let mut spi1 = Spi::new_blocking_txonly(p.SPI0, spi1_clk, spi1_mosi, addressable_led_config);
    // let buf = [0x00, 0x00, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB];
    // let buf = [0x00, 0x00, 0xE0, 0xE0, 0xE0]; // 0xE0 is logical 0 0xFC is logical 1
    // 300 us is the minimum for the reset period, the datasheets says 50us >:(
    // let buf = [0x00, 0x00, L0, L0, L0, L0, L0, L0, L0, L0, L1, L1, L1, L1, L1, L1, L1, L1, L0, L0, L0, L0, L0, L0, L0, L0];
    let buf = [0x00, 0x00, L1, L1, L1, L1, L1, L1, L1, L0, L0, L0, L0, L0, L0, L0, L0, L1, L1, L1, L1, L1, L1, L1, L1, L1];
    let buf2 = [0x00, 0x00, L0, L0, L0, L0, L0, L0, L0, L0, L1, L1, L1, L1, L1, L1, L1, L1, L1, L1, L1, L1, L1, L1, L1, L1];
    // spi1.blocking_write(&buf).unwrap();
    /*
        Setup Section End
     */

    /*
        Loop Section Begin (run continuously)
     */
    // Putting the loop in an asynchronous function lets us run the loop and the logger at the same time
    let echo_fut = async {
        loop {
            log::info!("Teal");
            spi1.blocking_write(&buf).unwrap();
            Timer::after_millis(300).await;
            log::info!("Magenta");
            spi1.blocking_write(&buf2).unwrap();
            Timer::after_millis(300).await;
            // for i in 0..360 {
            //     let rgb = hsl_to_rgb(i as f32, 1.0, 0.5);
            //     log::info!("Red: {}, Green: {}, Blue: {}", rgb.0, rgb.1, rgb.2);
            //     set_red(map32(rgb.0 as i32, 0, 255, DEFAULT_BOT as i32, DEFAULT_TOP as i32) as u16);
            //     set_green(map32(rgb.1 as i32, 0, 255, DEFAULT_BOT as i32, DEFAULT_TOP as i32) as u16);
            //     set_blue(map32(rgb.2 as i32, 0, 255, DEFAULT_BOT as i32, DEFAULT_TOP as i32) as u16);
            //     Timer::after_millis(DEFAULT_DELAY).await;
            // }
            
            // for i in DEFAULT_BOT..DEFAULT_TOP {
            //     // log::info!("current i: {}", i);
            //     set_red(i);
            //     set_green(i);
            //     set_blue(i);
            //     Timer::after_micros(DEFAULT_DELAY).await;
            // }


            // for i in (DEFAULT_BOT..DEFAULT_TOP).rev() {
            //     // log::info!("current i: {}", i);
            //     set_red(i);
            //     set_green(i);
            //     set_blue(i);
            //     Timer::after_micros(DEFAULT_DELAY).await;
            // }

        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(echo_fut, log_fut)).await;

    /*
        Loop Section End
     */
}

fn map32(input: i32, input_start: i32, input_end: i32, output_start: i32, output_end: i32) -> i32 {
    let slope = (output_end - output_start) / (input_end - input_start);
    output_start + slope * (input - input_start)
}


fn abs32(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & (i32::MAX as u32))
}


fn hue_to_rgb(chroma: f32, x: f32, hue_prime: f32) -> (f32, f32, f32) {
    if hue_prime >= 0.0 && hue_prime <= 1.0 {
        return (chroma, x, 0.0)
    } else if hue_prime >= 1.0 && hue_prime <= 2.0 {
        return (x, chroma, 0.0)
    } else if hue_prime > 2.0 && hue_prime <= 3.0 {
        return (0.0, chroma, x)
    } else if hue_prime > 3.0 && hue_prime <= 4.0 {
        return (0.0, x, chroma)
    } else if hue_prime > 4.0 && hue_prime <= 5.0 {
        return (x, 0.0, chroma)
    } else if hue_prime > 5.0 && hue_prime <= 6.0 {
        return (chroma, 0.0, x)
    }

    (0.0, 0.0, 0.0)
}

// Takes in an hsl value and returns an rgb value from it
fn hsl_to_rgb(hue: f32, saturation: f32, lumincance: f32) -> (u8, u8, u8) {
    // Initialize all values to 0
    let mut red:u8 = 0;
    let mut green:u8 = 0;
    let mut blue:u8 = 0;

    if saturation == 0.0 {
        red = (lumincance * 255.0) as u8;
        green = (lumincance * 255.0) as u8;
        blue = (lumincance * 255.0) as u8;
        // log::info!("S = 0 == Red: {} Green: {} Blue: {}", red, green, blue);
    } else {
        let chroma: f32 = (1.0 - abs32(2.0 * lumincance - 1.0)) * saturation as f32;
        let hue_prime: f32 = hue / 60.0;
        let x: f32 = chroma * (1.0 - abs32(hue_prime % 2.0 - 1.0));

        let rgb_values:(f32, f32, f32) = hue_to_rgb(chroma, x, hue_prime);

        let m = lumincance - (chroma / 2.0);

        red = ((rgb_values.0 + m) * 255.0) as u8;
        green = ((rgb_values.1 + m) * 255.0) as u8;
        blue = ((rgb_values.2 + m) * 255.0) as u8;
        // log::info!("S != 0 == Red: {} Green: {} Blue: {}", red, green, blue);
    }
    // log::info!("At End == Red: {} Green: {} Blue: {}", red, green, blue);
    (red, green, blue)
}