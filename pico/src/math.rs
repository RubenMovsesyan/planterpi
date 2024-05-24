pub fn map32(input: i32, input_start: i32, input_end: i32, output_start: i32, output_end: i32) -> i32 {
    let slope = (output_end - output_start) / (input_end - input_start);
    output_start + slope * (input - input_start)
}


pub fn abs32(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & (i32::MAX as u32))
}



pub mod color_math {
    use super::*;

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
    pub fn hsl_to_rgb(hue: f32, saturation: f32, lumincance: f32) -> u32 {
        // Initialize all values to 0
        let mut red:u32 = 0;
        let mut green:u32 = 0;
        let mut blue:u32 = 0;

        if saturation == 0.0 {
            red = (lumincance * 255.0) as u32;
            green = (lumincance * 255.0) as u32;
            blue = (lumincance * 255.0) as u32;
            // log::info!("S = 0 == Red: {} Green: {} Blue: {}", red, green, blue);
        } else {
            let chroma: f32 = (1.0 - abs32(2.0 * lumincance - 1.0)) * saturation as f32;
            let hue_prime: f32 = hue / 60.0;
            let x: f32 = chroma * (1.0 - abs32(hue_prime % 2.0 - 1.0));

            let rgb_values:(f32, f32, f32) = hue_to_rgb(chroma, x, hue_prime);

            let m = lumincance - (chroma / 2.0);

            red = ((rgb_values.0 + m) * 255.0) as u32;
            green = ((rgb_values.1 + m) * 255.0) as u32;
            blue = ((rgb_values.2 + m) * 255.0) as u32;
            // log::info!("S != 0 == Red: {} Green: {} Blue: {}", red, green, blue);
        }
        // log::info!("At End == Red: {} Green: {} Blue: {}", red, green, blue);
        let mut ret: u32 = 0;
        ret |= red << 16;
        ret |= green << 8;
        ret |= blue;
        ret
    }


    pub fn u32_to_rgb(rgb: u32) -> (u8, u8, u8) {
        let red =   ((rgb & 0xFF0000) >> 16) as u8;
        let green = ((rgb & 0x00FF00) >> 8) as u8;
        let blue =  ((rgb & 0xFF)) as u8;
        
        (red, green, blue)
    }
}