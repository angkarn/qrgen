use qrcode_generator::QrCodeEcc;
use rust_text_draw::image::{open, DynamicImage, Rgba};
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
    pub template_draw: Option<String>,
    pub font_size: f32,
    pub reduce_font_size: u32,
    pub font_db: fontdb::Database,
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
    content: String,
    opt: GenerateImageOptions,
) -> Result<ResultGenerateImage, String> {
    // Create a new image with additional space at the top
    let mut new_image = if opt.base_image.is_some() {
        open(opt.base_image.unwrap()).unwrap()
    } else {
        let mut temp_new_image = DynamicImage::new_rgba8(opt.image_width, opt.image_height);

        // Set widget color
        let fill_color_rgba: [u8; 4] = u32::from_str_radix(opt.fill_color.as_str(), 16)
            .unwrap()
            .to_be_bytes();

        // fill bg base image
        for y in 0..temp_new_image.height() as u32 {
            for x in 0..temp_new_image.width() as u32 {
                temp_new_image.put_pixel(x, y, Rgba(fill_color_rgba))
            }
        }
        temp_new_image
    };

    // Generate and draw QR
    if opt.qr_size != 0 {
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

        // Copy the QR code image onto the new image
        let qr_color_0: [u8; 4] = u32::from_str_radix(opt.qr_color.0.as_str(), 16)
            .unwrap()
            .to_be_bytes();

        let qr_color_1: [u8; 4] = u32::from_str_radix(opt.qr_color.1.as_str(), 16)
            .unwrap()
            .to_be_bytes();

        for x in 0..qr_code_buffer.width() {
            for y in 0..qr_code_buffer.height() {
                let pixel_value = qr_code_buffer.get_pixel(x, y);
                let new_image_pixel = new_image.get_pixel(opt.pos_qr_x + x, opt.pos_qr_y + y);
                if pixel_value[0] == 0 {
                    let color = get_alpha_color(new_image_pixel, qr_color_1);
                    new_image.put_pixel(opt.pos_qr_x + x, opt.pos_qr_y + y, Rgba(color));
                } else {
                    let color = get_alpha_color(new_image_pixel, qr_color_0);
                    new_image.put_pixel(opt.pos_qr_x + x, opt.pos_qr_y + y, Rgba(color));
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
    let text = opt.template_draw.clone().unwrap();

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
