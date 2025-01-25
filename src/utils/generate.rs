use qrcode_generator::QrCodeEcc;
use rust_text_draw::image::{DynamicImage, Rgba};
use rust_text_draw::{draw_text, Widget};
use rust_text_draw::{fontdb, FontSystem, GenericImage, SwashCache};

pub struct ResultGenerateImage {
    pub image_buffer: DynamicImage,
    pub reduce_font_size: bool,
    pub draw_out_pixel: bool,
}

pub struct GenerateImageOptions {
    pub image_width: u32,
    pub image_height: u32,
    pub qr_size: u32,
    pub error_correction_level: String,
    pub pos_qr_x: u32,
    pub pos_qr_y: u32,
    pub template_text_render: Option<String>,
    pub font_size: u32,
    pub reduce_font_size: u32,
    pub font_db: fontdb::Database,
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
    let mut new_image = DynamicImage::new_rgba8(opt.image_width, opt.image_height);

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
                new_image.put_pixel(opt.pos_qr_x + x, opt.pos_qr_y + y, Rgba([0, 0, 0, 255]));
            } else {
                new_image.put_pixel(
                    opt.pos_qr_x + x,
                    opt.pos_qr_y + y,
                    Rgba([pixel_value, pixel_value, pixel_value, 255]),
                );
            }
        }
    }

    if opt.template_text_render.is_none() {
        return Ok(ResultGenerateImage {
            image_buffer: new_image,
            reduce_font_size: false,
            draw_out_pixel: false,
        });
    }

    // Process draw text to image
    let text = opt.template_text_render.clone().unwrap();

    let widgets = json5::from_str::<Vec<Widget>>(&text).unwrap();
    // println!("deserialized = {:#?}", widgets);

    let mut swash_cache = SwashCache::new();
    let text_layout_width = new_image.width();
    let text_layout_height = new_image.height();

    let mut font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), opt.font_db);

    // for wait to set and return result
    let reduce_font_size = false;

    let result_draw_text = draw_text(
        &mut swash_cache,
        &mut font_system,
        &mut new_image,
        0.0,
        0.0,
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
