use image::{DynamicImage, GenericImage, Rgba};

pub trait FillPixels {
    fn fill_pixels(&mut self, x: u32, y: u32, width: u32, height: u32, color: Rgba<u8>);
}

impl FillPixels for DynamicImage {
    fn fill_pixels(&mut self, x: u32, y: u32, width: u32, height: u32, color: Rgba<u8>) {
        for x_iter in x..(x + width) {
            for y_iter in y..(y + height) {
                self.put_pixel(x_iter, y_iter, color);
            }
        }
    }
}