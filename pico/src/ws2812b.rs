use embassy_rp::spi;

const DRIVER_FREQUENCY: u32 = 8_500_000;
pub const INFO_SIZE: usize = 24;
const LOGIC_0: u16 = 0xE0;
const LOGIC_1: u16 = 0xFC;
pub const NUM_LEDS: usize = 12;
pub const LED_INFO_SIZE: usize = 2 + (INFO_SIZE * NUM_LEDS);


pub fn get_addressable_led_config() -> spi::Config {
    let mut config = spi::Config::default();
    config.frequency = DRIVER_FREQUENCY;
    config
}

fn rgb_to_logic_buffer(color_value: u32) -> [u16; INFO_SIZE] {
    // Get the red logic
    let mut mask: u32 = 0x00800000;
    let mut logic_buffer_bits: [u16; INFO_SIZE] = [0; INFO_SIZE];

    for i in 0..8 {
        if color_value & mask > 0 {
            logic_buffer_bits[(1 * 8) + i] = LOGIC_1;
        } else {
            logic_buffer_bits[(1 * 8) + i] = LOGIC_0;
        }
        mask >>= 1;
    }
    
    // Get the green logic
    for i in 0..8 {
        if color_value & mask > 0 {
            logic_buffer_bits[i] = LOGIC_1;
        } else {
            logic_buffer_bits[i] = LOGIC_0;
        }
        mask >>= 1;
    }

    // Get the blue logic
    for i in 0..8 {
        if color_value & mask > 0 {
            logic_buffer_bits[(2 * 8) + i] = LOGIC_1;
        } else {
            logic_buffer_bits[(2 * 8) + i] = LOGIC_0;
        }
        mask >>= 1;
    }

    logic_buffer_bits
}

// Count must be the number of leds * INFO_SIZE
pub fn generate_addressable_led_buffer<const COUNT: usize>(color_values: &[u32]) -> [u16; COUNT] {
    let mut addressable_led_buffer: [u16; COUNT] = [0; COUNT];

    let mut index: u32 = 2;
    for color in color_values.iter() {
        let color_buffer = rgb_to_logic_buffer(*color);

        for i in 0..INFO_SIZE {
            addressable_led_buffer[index as usize] = color_buffer[i];
            index += 1;
        }
    }
    addressable_led_buffer
}