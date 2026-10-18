use qrcode_generator::QrCodeEcc;
use rust_text_draw::image::{open, DynamicImage, Rgba, RgbaImage};
use rust_text_draw::{draw_text, GenericImageView, Widget};
use rust_text_draw::{fontdb, FontSystem, GenericImage, SwashCache};

pub struct ResultGenerateImage {
    pub image_buffer: DynamicImage,
    pub reduce_font_size: bool,
    pub draw_out_pixel: bool,
}

pub struct GenerateImageOptions {
    pub qr_color: (String, String),
    pub base_image: Option<String>,
    pub fill_color: String,
    pub image_width: u32,
    pub image_height: u32,
    pub qr_size: u32,
    pub error_correction_level: String,
    pub pos_qr_x: u32,
    pub pos_qr_y: u32,
    pub template_draw: Option<Vec<Widget>>,
    pub font_size: f32,
    pub reduce_font_size: u32,
}

/// Font/glyph caches reused across images.
/// `FontSystem` and `SwashCache` are not `Sync`, so for parallel work keep one per thread.
pub struct TextCache {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
}

impl TextCache {
    pub fn new(font_db: fontdb::Database) -> Self {
        Self {
            font_system: FontSystem::new_with_locale_and_db("en-US".to_string(), font_db),
            swash_cache: SwashCache::new(),
        }
    }
}

pub fn get_alpha_color(base_color: Rgba<u8>, color: [u8; 4]) -> [u8; 4] {
    let new_alpha = color[3] as f32 / 255.0;
    let base_alpha = base_color[3] as f32 / 255.0;

    let scale =
        |dc: u8, bc: u8| (dc as f32 * new_alpha) + (bc as f32 * base_alpha * (1.0 - new_alpha));

    let r = scale(color[0], base_color[0]) as u8;
    let g = scale(color[1], base_color[1]) as u8;
    let b = scale(color[2], base_color[2]) as u8;
    let alpha = (255.0 * (new_alpha + base_alpha * (1.0 - new_alpha))) as u8;
    return [r, g, b, alpha];
}

pub fn generate_image(
    content: Option<String>,
    opt: GenerateImageOptions,
    text_cache: &mut TextCache,
) -> Result<ResultGenerateImage, String> {
    // Create a new image with additional space at the top
    let mut new_image = if opt.base_image.is_some() {
        open(opt.base_image.unwrap()).unwrap()
    } else {
        // Set widget color
        let fill_color_rgba: [u8; 4] = u32::from_str_radix(opt.fill_color.as_str(), 16)
            .unwrap()
            .to_be_bytes();

        // fill bg base image
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            opt.image_width,
            opt.image_height,
            Rgba(fill_color_rgba),
        ))
    };

    // Generate and draw QR
    if opt.qr_size != 0 && content.is_some() {
        // set qr error correction level
        let ecc = match opt.error_correction_level.as_str() {
            "l" => QrCodeEcc::Low,
            "m" => QrCodeEcc::Medium,
            "q" => QrCodeEcc::Quartile,
            "h" => QrCodeEcc::High,
            _ => QrCodeEcc::Medium,
        };

        // Generate a QR code and convert it to ImageBuffer
        let qr_code_buffer = qrcode_generator::to_image_buffer(content.unwrap(), ecc, opt.qr_size as usize)
            .expect("Failed to generate QR code");

        // Copy the QR code image onto the new image
        let qr_color_0: [u8; 4] = u32::from_str_radix(opt.qr_color.0.as_str(), 16)
            .unwrap()
            .to_be_bytes();

        let qr_color_1: [u8; 4] = u32::from_str_radix(opt.qr_color.1.as_str(), 16)
            .unwrap()
            .to_be_bytes();

        // Row-major loop (matches memory layout). Blending only depends on
        // (base pixel, qr color), so remember the last result to skip repeated math.
        let mut last_blend: [Option<(Rgba<u8>, [u8; 4])>; 2] = [None, None];
        let mut blend = |base: Rgba<u8>, is_dark: bool| -> [u8; 4] {
            let (slot, qr_color) = if is_dark {
                (&mut last_blend[1], qr_color_1)
            } else {
                (&mut last_blend[0], qr_color_0)
            };
            match slot {
                Some((b, c)) if *b == base => *c,
                _ => {
                    let c = get_alpha_color(base, qr_color);
                    *slot = Some((base, c));
                    c
                }
            }
        };

        for y in 0..qr_code_buffer.height() {
            for x in 0..qr_code_buffer.width() {
                let is_dark = qr_code_buffer.get_pixel(x, y)[0] == 0;
                let (px, py) = (opt.pos_qr_x + x, opt.pos_qr_y + y);
                match &mut new_image {
                    // Fast path: direct buffer access (skips DynamicImage conversion)
                    DynamicImage::ImageRgba8(buf) => {
                        let pixel = buf.get_pixel_mut(px, py);
                        *pixel = Rgba(blend(*pixel, is_dark));
                    }
                    image => {
                        let color = blend(image.get_pixel(px, py), is_dark);
                        image.put_pixel(px, py, Rgba(color));
                    }
                }
            }
        }
    }

    if opt.template_draw.is_none() {
        return Ok(ResultGenerateImage {
            image_buffer: new_image,
            reduce_font_size: false,
            draw_out_pixel: false,
        });
    }

    // Process draw text to image

    let widgets = opt.template_draw.unwrap();
    // println!("deserialized = {:#?}", widgets);

    let text_layout_width = new_image.width();
    let text_layout_height = new_image.height();

    // for wait to set and return result
    let reduce_font_size = false;

    let result_draw_text = draw_text(
        &mut text_cache.swash_cache,
        &mut text_cache.font_system,
        &mut new_image,
        0.0,
        0.0,
        text_layout_width as f32,
        text_layout_height as f32,
        widgets,
        opt.font_size as f32,
        &"000000".to_string(),
        true,
    );

    match result_draw_text {
        Ok(r) => Ok(ResultGenerateImage {
            image_buffer: new_image,
            reduce_font_size,
            draw_out_pixel: r.count_pixel_out > 0,
        }),
        Err(e) => Err(format!("Unsuccess: {}", e)),
    }
}
