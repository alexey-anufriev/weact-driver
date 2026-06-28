use crate::{DisplaySpec, Orientation, Rgb565};

/// Image buffer stored in program memory before it is sent to the display.
///
/// Pixels are stored in row-major order:
///
/// ```text
/// index = y * width + x
/// ```
///
/// Rows are serialized in the same order when uploading to the display.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Framebuffer {
    width: u16,
    height: u16,
    pixels: Vec<Rgb565>,
}

impl Framebuffer {
    /// Creates a framebuffer for `spec` and `orientation`.
    pub fn new_for_display(spec: &DisplaySpec, orientation: Orientation) -> Self {
        let (width, height) = orientation.dimensions(spec);
        Self {
            width,
            height,
            pixels: vec![Rgb565::BLACK; width as usize * height as usize],
        }
    }

    /// Creates a portrait framebuffer for `spec`.
    pub fn new_portrait(spec: &DisplaySpec) -> Self {
        Self::new_for_display(spec, Orientation::Portrait)
    }

    /// Creates a landscape framebuffer for `spec`.
    pub fn new_landscape(spec: &DisplaySpec) -> Self {
        Self::new_for_display(spec, Orientation::Landscape)
    }

    /// Framebuffer width in pixels.
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Framebuffer height in pixels.
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// Sets every pixel to `color`.
    pub fn clear(&mut self, color: Rgb565) {
        self.pixels.fill(color);
    }

    /// Writes one pixel if `(x, y)` is inside the framebuffer.
    ///
    /// Out-of-bounds writes are ignored, which makes clipped drawing code straightforward.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Rgb565) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = color;
        }
    }

    /// Reads one pixel, or returns `None` if `(x, y)` is out of bounds.
    ///
    /// Use this for bounds-checked inspection of framebuffer contents.
    pub fn get_pixel(&self, x: u16, y: u16) -> Option<Rgb565> {
        self.index(x, y).map(|index| self.pixels[index])
    }

    /// Fills a rectangle, clipped to the framebuffer.
    ///
    /// Rectangles extending past the right or bottom edge draw only their visible portion.
    /// Empty rectangles draw nothing.
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Rgb565) {
        let x_end = x.saturating_add(w).min(self.width);
        let y_end = y.saturating_add(h).min(self.height);

        for yy in y..y_end {
            for xx in x..x_end {
                self.set_pixel(xx, yy, color);
            }
        }
    }

    /// Serializes the framebuffer as little-endian RGB565 bytes.
    ///
    /// The returned vector has exactly `width * height * 2` bytes.
    pub fn as_rgb565_le_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 2);
        for pixel in &self.pixels {
            bytes.extend_from_slice(&pixel.to_le_bytes());
        }
        bytes
    }

    /// Converts `(x, y)` into the internal vector index.
    ///
    /// Keeping this in one place gives all pixel operations the same bounds behavior.
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y as usize * self.width as usize + x as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::Framebuffer;
    use crate::{Orientation, Rgb565, WEACT_FS_096_80X160, WEACT_FS_V1_320X480};

    #[test]
    fn creates_orientation_sized_framebuffers() {
        let portrait = Framebuffer::new_portrait(&WEACT_FS_096_80X160);
        assert_eq!((portrait.width(), portrait.height()), (80, 160));

        let landscape = Framebuffer::new_landscape(&WEACT_FS_096_80X160);
        assert_eq!((landscape.width(), landscape.height()), (160, 80));

        let v1_portrait = Framebuffer::new_portrait(&WEACT_FS_V1_320X480);
        assert_eq!((v1_portrait.width(), v1_portrait.height()), (320, 480));

        let v1_landscape = Framebuffer::new_landscape(&WEACT_FS_V1_320X480);
        assert_eq!((v1_landscape.width(), v1_landscape.height()), (480, 320));

        let fs096_portrait =
            Framebuffer::new_for_display(&WEACT_FS_096_80X160, Orientation::Portrait);
        assert_eq!((fs096_portrait.width(), fs096_portrait.height()), (80, 160));
    }

    #[test]
    fn indexes_pixels_row_major() {
        let mut fb = Framebuffer::new_landscape(&WEACT_FS_096_80X160);
        fb.set_pixel(2, 1, Rgb565::RED);

        let bytes = fb.as_rgb565_le_bytes();
        let red_offset = ((fb.width() as usize) + 2) * 2;

        assert_eq!(fb.get_pixel(2, 1), Some(Rgb565::RED));
        assert_eq!(&bytes[0..8], &[0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&bytes[red_offset..red_offset + 2], &[0x00, 0xf8]);
    }

    #[test]
    fn out_of_bounds_pixel_access_is_ignored() {
        let mut fb = Framebuffer::new_landscape(&WEACT_FS_096_80X160);
        fb.set_pixel(fb.width(), 0, Rgb565::WHITE);
        fb.set_pixel(0, fb.height(), Rgb565::WHITE);

        assert_eq!(fb.get_pixel(fb.width(), 0), None);
        assert_eq!(fb.get_pixel(0, fb.height()), None);
        assert!(fb.as_rgb565_le_bytes().iter().all(|byte| *byte == 0));
    }

    #[test]
    fn fill_rect_clips_to_framebuffer() {
        let mut fb = Framebuffer::new_landscape(&WEACT_FS_096_80X160);
        let last_x = fb.width() - 1;
        let last_y = fb.height() - 1;

        fb.fill_rect(last_x, last_y, 5, 5, Rgb565::BLUE);

        assert_eq!(fb.get_pixel(0, 0), Some(Rgb565::BLACK));
        assert_eq!(fb.get_pixel(last_x - 1, last_y), Some(Rgb565::BLACK));
        assert_eq!(fb.get_pixel(last_x, last_y), Some(Rgb565::BLUE));
    }
}
