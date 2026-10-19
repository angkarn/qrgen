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
    // Fill color of a new image (None when drawing on a base image)
    let fill_color_rgba: Option<[u8; 4]> = match opt.base_image {
        Some(_) => None,
        None => Some(
            u32::from_str_radix(opt.fill_color.as_str(), 16)
                .unwrap()
                .to_be_bytes(),
        ),
    };

    // Create a new image with additional space at the top
    let mut new_image = match fill_color_rgba {
        None => open(opt.base_image.unwrap()).unwrap(),
        // fill bg base image
        Some(fill) => DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            opt.image_width,
            opt.image_height,
            Rgba(fill),
        )),
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

        // Generate the QR module matrix (rows[y][x])
        let modules = qrcode_generator::to_matrix(content.unwrap(), ecc)
            .expect("Failed to generate QR code");

        // Same layout as qrcode_generator::to_image_buffer: 1-module quiet zone,
        // square modules of `point_size` pixels, centered in `qr_size`
        let qr_size = opt.qr_size as usize;
        let point_size = qr_size / (modules.len() + 2);
        assert!(point_size > 0, "Failed to generate QR code: QR size is too small");
        let margin = (qr_size - point_size * modules.len()) / 2;
        let module_at = |p: usize| -> Option<usize> {
            (p >= margin && p < margin + point_size * modules.len())
                .then(|| (p - margin) / point_size)
        };
        // Dark flags of one pixel row (identical for every pixel row of a module row)
        let row_flags = |module_row: Option<usize>| -> Vec<bool> {
            (0..qr_size)
                .map(|x| match (module_row, module_at(x)) {
                    (Some(r), Some(c)) => modules[r][c],
                    _ => false,
                })
                .collect()
        };

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

        let (pos_x, pos_y) = (opt.pos_qr_x as usize, opt.pos_qr_y as usize);
        assert!(
            pos_x + qr_size <= new_image.width() as usize
                && pos_y + qr_size <= new_image.height() as usize,
            "QR is out of image bounds"
        );

        let mut cached_module_row: Option<Option<usize>> = None;
        let mut flags: Vec<bool> = Vec::new();

        match (&mut new_image, fill_color_rgba) {
            // Fast path: on a new image every pixel under the QR is the fill color, so
            // build each module row once and copy it into the buffer
            (DynamicImage::ImageRgba8(buf), Some(fill)) => {
                let light = blend(Rgba(fill), false);
                let dark = blend(Rgba(fill), true);
                let img_w = buf.width() as usize;
                let raw: &mut [u8] = buf;
                let mut row_bytes: Vec<u8> = Vec::with_capacity(qr_size * 4);
                for y in 0..qr_size {
                    let module_row = module_at(y);
                    if cached_module_row != Some(module_row) {
                        row_bytes.clear();
                        for &is_dark in row_flags(module_row).iter() {
                            row_bytes.extend_from_slice(if is_dark { &dark } else { &light });
                        }
                        cached_module_row = Some(module_row);
                    }
                    let start = ((pos_y + y) * img_w + pos_x) * 4;
                    raw[start..start + qr_size * 4].copy_from_slice(&row_bytes);
                }
            }
            // Base image: blend with each base pixel
            (image, _) => {
                for y in 0..qr_size {
                    let module_row = module_at(y);
                    if cached_module_row != Some(module_row) {
                        flags = row_flags(module_row);
                        cached_module_row = Some(module_row);
                    }
                    let py = (pos_y + y) as u32;
                    for (x, &is_dark) in flags.iter().enumerate() {
                        let px = (pos_x + x) as u32;
                        match &mut *image {
                            // Direct buffer access (skips DynamicImage conversion)
                            DynamicImage::ImageRgba8(buf) => {
                                let pixel = buf.get_pixel_mut(px, py);
                                *pixel = Rgba(blend(*pixel, is_dark));
                            }
                            other => {
                                let color = blend(other.get_pixel(px, py), is_dark);
                                other.put_pixel(px, py, Rgba(color));
                            }
                        }
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
