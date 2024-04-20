use ab_glyph::{FontVec, PxScale};
use imageproc::{
    drawing::{draw_text_mut, text_size},
    image::{ImageBuffer, Rgb},
};
use qrcode_generator::QrCodeEcc;
use std::fs::read;

static FONT_DEFAULT: &'static [u8] = include_bytes!("../NotoSansThai-Light.ttf");

pub struct ResultGenerateImage {
    pub image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pub reduce_bottom_text_size: Option<u32>,
    pub reduce_top_text_size: Option<u32>,
}

struct TextDraw {
    text: Box<str>,
    px_scale: PxScale,
    width: u32,
    height: u32,
}

struct PrepareTextDraw {
    text_draw_data: Vec<TextDraw>,
    height_texts_sum: u32,
    line_height: u32,
    reduce_text_size: Option<u32>,
}

fn prepare_text_draw(
    text: String,
    base_size: u32,
    percent_font_size: u32,
    font: &FontVec,
    text_line_space: u32,
    additional_space: u32,
    no_reduce_text_size: bool,
) -> Result<PrepareTextDraw, String> {
    let text_split = text.split("\\n");
    let mut text_draw_data: Vec<TextDraw> = Vec::new();

    let mut line_height = 0;
    let mut height_texts_sum = 0;
    let mut per_font_size = percent_font_size;
    // loop check and resize if text over size of image
    let result_loop_cal_size = 'loop_cal_size: loop {
        for (index_line, text) in text_split.clone().enumerate() {
            // Define text properties
            let px_scale = PxScale::from((base_size * per_font_size / 100) as f32);

            // Calculate the total text width
            let _text_size = text_size(px_scale, &font, &text);

            text_draw_data.push(TextDraw {
                text: text.into(),
                px_scale,
                width: _text_size.0,
                height: _text_size.1,
            });

            line_height = base_size * (per_font_size as u32 / 4 + text_line_space) / 100;

            // first line is no margin
            if index_line > 0 {
                height_texts_sum += line_height;
            }

            // Summary height from multiple line
            height_texts_sum += _text_size.1;

            // Check size of calculate scale text must be less than base image
            if _text_size.0 < base_size && height_texts_sum < additional_space {
                // continue next line
                continue;
            } else {
                if no_reduce_text_size {
                    break 'loop_cal_size Some("Text size is over base image. try run without flag `--nrts` no_reduce_text_size.");
                }

                // Reduce percent font size to next loop check
                per_font_size -= 1;

                // Reset data
                height_texts_sum = 0;
                text_draw_data = Vec::new();
                continue 'loop_cal_size;
            }
        }
        break 'loop_cal_size None;
    };

    // Handler error for result_loop_cal_size
    if result_loop_cal_size.is_some() {
        return Err(result_loop_cal_size.unwrap().to_owned());
    }

    // Create option for reduce_text_size result
    let mut reduce_text_size: Option<u32> = None;
    if per_font_size < percent_font_size {
        reduce_text_size = Some(per_font_size)
    }

    return Ok(PrepareTextDraw {
        text_draw_data,
        height_texts_sum,
        line_height,
        reduce_text_size,
    });
}

pub fn generate_image(
    content: String,
    size: u32,
    text_top: &String,
    text_bottom: &String,
    top_space: usize,
    bottom_space: usize,
    top_text_pos: &String,
    bottom_text_pos: &String,
    font_path: &Option<String>,
    font_size: usize,
    no_reduce_text_size: bool,
    add_text_line_space: u32,
    error_correction_level: &String,
) -> Result<ResultGenerateImage, String> {
    // set qr error correction level
    let ecc = match error_correction_level.as_str() {
        "l" => QrCodeEcc::Low,
        "m" => QrCodeEcc::Medium,
        "q" => QrCodeEcc::Quartile,
        "h" => QrCodeEcc::High,
        _ => QrCodeEcc::Medium,
    };

    // Generate a QR code and convert it to ImageBuffer
    let qr_code_buffer = qrcode_generator::to_image_buffer(content, ecc, size as usize)
        .expect("Failed to generate QR code");

    // Define the additional space to be added on top
    let cal_top_space = if !text_top.is_empty() {
        size * top_space as u32 / 100
    } else {
        0
    };

    // Define the additional space to be added on bottom
    let cal_bottom_space = if !text_bottom.is_empty() {
        size * bottom_space as u32 / 100
    } else {
        0
    };

    // Create a new image with additional space at the top
    let mut new_image = ImageBuffer::from_pixel(
        qr_code_buffer.width(),
        qr_code_buffer.height() + cal_top_space + cal_bottom_space,
        Rgb([255, 255, 255]),
    );

    // Copy the QR code image onto the new image
    for x in 0..qr_code_buffer.width() {
        for y in 0..qr_code_buffer.height() {
            let pixel_value = qr_code_buffer.get_pixel(x, y)[0];
            new_image.put_pixel(
                x,
                y + cal_top_space as u32,
                Rgb([pixel_value, pixel_value, pixel_value]),
            );
        }
    }

    // for wait to set and return result
    let mut reduce_top_text_size: Option<u32> = None;
    let mut reduce_bottom_text_size: Option<u32> = None;

    if !text_top.is_empty() || !text_bottom.is_empty() {
        // Get font data
        let font_data = if font_path.is_none() {
            Vec::<u8>::from(FONT_DEFAULT)
        } else {
            read(font_path.as_ref().unwrap()).expect("Error read font file")
        };

        let font = FontVec::try_from_vec(font_data).expect("Error constructing Font");

        let color = Rgb([0, 0, 0]);

        // Text Bottom
        if !text_bottom.is_empty() {
            let text_bottom_draw_data = match prepare_text_draw(
                text_bottom.to_string(),
                size,
                font_size as u32,
                &font,
                add_text_line_space,
                cal_bottom_space as u32,
                no_reduce_text_size,
            ) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };

            reduce_bottom_text_size = text_bottom_draw_data.reduce_text_size;

            process_draw_text(
                text_bottom_draw_data.text_draw_data,
                text_bottom_draw_data.height_texts_sum,
                text_bottom_draw_data.line_height,
                &mut new_image,
                cal_bottom_space,
                color,
                &font,
                bottom_text_pos.to_string(),
                "bottom".to_string(),
            )
        }

        // Text Top
        if !text_top.is_empty() {
            let text_top_draw_data = prepare_text_draw(
                text_top.to_string(),
                size,
                font_size as u32,
                &font,
                add_text_line_space,
                cal_top_space as u32,
                no_reduce_text_size,
            )
            .expect("Error: Prepare text draw (top)");

            reduce_top_text_size = text_top_draw_data.reduce_text_size;

            process_draw_text(
                text_top_draw_data.text_draw_data,
                text_top_draw_data.height_texts_sum,
                text_top_draw_data.line_height,
                &mut new_image,
                cal_top_space,
                color,
                &font,
                top_text_pos.to_string(),
                "top".to_string(),
            )
        }
    }

    Ok(ResultGenerateImage {
        image_buffer: new_image,
        reduce_top_text_size,
        reduce_bottom_text_size,
    })
}

// Draw text to image
fn process_draw_text(
    text_draw_data: Vec<TextDraw>,
    height_texts_sum: u32,
    line_height: u32,
    base_image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    side_space: u32,
    color: Rgb<u8>,
    font: &FontVec,
    text_pos: String,
    side: String,
) {
    let mut current_height_text_drawn: u32 = 0;
    let base_image_width = &base_image.width();
    let base_image_height = &base_image.height();

    for text_draw in text_draw_data {
        // Calculate position y relate of line
        let start_y = current_height_text_drawn;

        // Render text onto the image
        draw_text_mut(
            base_image,
            color,
            match text_pos.as_str() {
                "center" => {
                    // Calculate the x-coordinate to center the text
                    ((base_image_width - text_draw.width) / 2) as i32
                }
                _ => {
                    // Calculate the x-coordinate to center the text
                    ((base_image_width - text_draw.width) / 2) as i32
                }
            },
            match side.as_str() {
                "top" => {
                    (((side_space * 50 / 100) - height_texts_sum as u32 / 2) + start_y as u32)
                        as i32
                }
                "bottom" => {
                    let botton_y =
                        ((base_image_height - side_space - (base_image_height * 3 / 100))
                            + (side_space / 2 - (height_texts_sum as u32 / 2) + start_y as u32))
                            as i32;
                    botton_y
                }
                _ => {
                    (((side_space * 50 / 100) - height_texts_sum as u32 / 2) + start_y as u32)
                        as i32
                }
            },
            text_draw.px_scale,
            &font,
            &text_draw.text,
        );

        current_height_text_drawn += text_draw.height + line_height
    }
}
