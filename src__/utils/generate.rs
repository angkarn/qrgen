use qrcode_generator::QrCodeEcc;
use rust_text_draw::image::{DynamicImage, Rgba};
use rust_text_draw::{draw_text, Widgets};
use rust_text_draw::{fontdb, FontSystem, GenericImage, SwashCache};

// static FONT_DEFAULT: &'static [u8] = include_bytes!("../../NotoSansThai-Light.ttf");

pub struct ResultGenerateImage {
    pub image_buffer: DynamicImage,
    pub reduce_font_size: bool,
    pub draw_out_pixel: bool,
}

pub struct GenerateImageOptions {
    pub qr_size: u32,
    pub error_correction_level: String,
    pub left_space: u32,
    pub top_space: u32,
    pub right_space: u32,
    pub bottom_space: u32,
    pub text_render_template: Option<String>,
    pub font_size: u32,
    pub reduce_font_size: u32,
    pub text_line_height: f32,
    pub locale_and_db: (String, fontdb::Database),
}

pub fn generate_image(
    content: String,
    opt: GenerateImageOptions,
) -> Result<ResultGenerateImage, String> {
    // set qr error correction level
    let ecc = match opt.error_correction_level.as_str() {
        "l" => QrCodeEcc::Low,
        "m" => QrCodeEcc::Medium,
        "q" => QrCodeEcc::Quartile,
        "h" => QrCodeEcc::High,
        _ => QrCodeEcc::Medium,
    };

    // Generate a QR code and convert it to ImageBuffer
    let qr_code_buffer = qrcode_generator::to_image_buffer(content, ecc, opt.qr_size as usize)
        .expect("Failed to generate QR code");

    // Create a new image with additional space at the top
    let mut new_image = DynamicImage::new_rgba8(
        qr_code_buffer.width() + opt.left_space + opt.right_space,
        qr_code_buffer.height() + opt.top_space + opt.bottom_space,
    );

    // fill bg base image
    for y in 0..new_image.height() as u32 {
        for x in 0..new_image.width() as u32 {
            new_image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // Copy the QR code image onto the new image
    for x in 0..qr_code_buffer.width() {
        for y in 0..qr_code_buffer.height() {
            let pixel_value = qr_code_buffer.get_pixel(x, y)[0];
            if pixel_value == 0 {
                new_image.put_pixel(opt.left_space + x, opt.top_space + y, Rgba([0, 0, 0, 255]));
            } else {
                new_image.put_pixel(
                    opt.left_space + x,
                    opt.top_space + y,
                    Rgba([pixel_value, pixel_value, pixel_value, 255]),
                );
            }
        }
    }

    // Process draw text to image
    let text = opt.text_render_template.clone().unwrap();

    let widgets = json5::from_str::<Vec<Widgets>>(&text).unwrap();
    // println!("deserialized = {:#?}", widgets);

    let mut swash_cache = SwashCache::new();

    let text_layout_width = new_image.width();
    let text_layout_height = new_image.height();

    let mut font_system =
        FontSystem::new_with_locale_and_db(opt.locale_and_db.0, opt.locale_and_db.1);

    // for wait to set and return result
    let reduce_font_size = false;

    let result_draw_text = draw_text(
        &mut swash_cache,
        &mut font_system,
        &mut new_image,
        0,
        0,
        text_layout_width,
        text_layout_height,
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
