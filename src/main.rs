use ab_glyph::{FontVec, PxScale};
use clap::{Parser, Subcommand};
use csv::Error as ErrorCsv;
use image::ImageBuffer;
use imageproc::image::Rgba;
use imageproc::{
    drawing::{draw_text_mut, text_size},
    image,
};
use qrcode_generator::QrCodeEcc;
use std::error::Error;
use std::fs::{self, create_dir_all, read};

static FONT_DEFAULT: &'static [u8] = include_bytes!("NotoSansThai-Light.ttf");

/// QR Code Generator Tools
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate qrcode from content
    Gen(GenArg),

    /// Generate qrcode from a file of list content (csv format)
    From(FromArg),
}

#[derive(Parser, Debug)]
struct CommonArg {
    /// Format output (console|png)
    #[clap(short = 'f', long, default_value = "console")]
    format: String,

    /// Size of image
    #[clap(short = 's', long, default_value = "1024")]
    size: u32,

    /// Output directory
    #[clap(short = 'o', long, default_value = "output")]
    outdir: String,

    /// Size of top space (percent of qr size)
    #[clap(long = "ts", default_value = "15")]
    top_space: usize,

    /// Size of bottom space (percent of qr size)
    #[clap(long = "bs", default_value = "15")]
    bottom_space: usize,

    /// Positional of text top
    #[clap(long = "ttp", default_value = "center", hide = true)]
    top_text_pos: String,

    /// Positional of text bottom
    #[clap(long = "btp", default_value = "center", hide = true)]
    bottom_text_pos: String,

    /// Path of font file
    #[clap(long = "fp")]
    font_path: Option<String>,

    /// Font size (percentage)
    #[clap(long = "fs", default_value = "10")]
    font_size: usize,

    /// Add text line space (percentage)
    #[clap(long = "atls", default_value = "0")]
    add_text_line_space: u32,

    /// Flag to ignore auto reduce text size
    #[clap(long = "nrts")]
    no_reduce_text_size: bool,
}

#[derive(Parser, Debug)]
#[command(
    after_help = "TEMPLATE: Can be use {{INDEX_COLUMN}} to replace from data (Starting at 0). eg. `Hello {{1}}` is replace {{1}} to data of index 1 on row."
)]
struct FromArg {
    /// Path file of list content
    path: String,

    /// Template content
    #[clap(short = 't', long = "tc", default_value = "{{0}}")]
    template_content: String,

    /// Template for text on top.
    #[clap(long = "ttt", default_value = "")]
    template_text_top: String,

    /// Template for text on bottom.
    #[clap(long = "ttb", default_value = "")]
    template_text_bottom: String,

    /// Template filename.
    #[clap(long = "tfn", default_value = "{{0}}")]
    template_filename: String,

    #[command(flatten)]
    common_arg: CommonArg,
}

#[derive(Parser, Debug)]
struct GenArg {
    /// Content to generate qrcode
    content: String,

    /// Text on top of image
    #[clap(short = 't', long, default_value = "")]
    top_text: String,

    /// Text on bottom of image
    #[clap(short = 'b', long, default_value = "")]
    bottom_text: String,

    #[command(flatten)]
    common_arg: CommonArg,
}

struct ResultGenerateImage {
    image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    reduce_bottom_text_size: Option<u32>,
    reduce_top_text_size: Option<u32>,
}

fn main() {
    let args = Args::parse();

    // println!("{:?}", args);

    match &args.command {
        Command::Gen(state) => handle_gen_command(state),
        Command::From(state) => handle_from_command(state),
    }
}

fn handle_gen_command(gen_opt: &GenArg) {
    match gen_opt.common_arg.format.as_str() {
        "console" => generate_console(&gen_opt.content),
        "png" => {
            let _ = create_dir_all(&gen_opt.common_arg.outdir.to_string())
                .expect("Cannot create output directory!");

            println!("\n\nGenerate Image...");

            let result = generate_image(
                gen_opt.content.clone(),
                gen_opt.common_arg.size,
                &gen_opt.top_text,
                &gen_opt.bottom_text,
                gen_opt.common_arg.top_space,
                gen_opt.common_arg.bottom_space,
                &gen_opt.common_arg.top_text_pos,
                &gen_opt.common_arg.bottom_text_pos,
                &gen_opt.common_arg.font_path,
                gen_opt.common_arg.font_size,
                gen_opt.common_arg.no_reduce_text_size,
                gen_opt.common_arg.add_text_line_space,
            );

            handler_generate_image_result(
                result,
                format!("{}/{}.png", gen_opt.common_arg.outdir, "qr"),
            )
        }
        _ => {}
    }
}

fn handle_from_command(from_opt: &FromArg) {
    let list_data = process_file(&from_opt.path).expect("Error processing file");

    println!("\nSample data processed:");
    print!("> {:?}", list_data.get(0).expect("No data after process"));

    println!("\n\nGenerate Images...");

    match from_opt.common_arg.format.as_str() {
        "console" => generate_list_console(list_data, from_opt),

        "png" => generate_list_image(list_data, from_opt),
        _ => {
            eprintln!("Can't found this format!")
        }
    }
}

fn generate_console(content: &String) {
    println!("{}", content);
    let result: Vec<Vec<bool>> = qrcode_generator::to_matrix(content, QrCodeEcc::Low).unwrap();
    print_qr(&result);
    println!();
}

fn generate_list_console(list_data: Vec<Vec<String>>, from_opt: &FromArg) {
    for row in list_data {
        let mut content: String = "".to_string();
        for (index_col, col) in row.into_iter().enumerate() {
            content = from_opt
                .template_content
                .replace(&String::from(format!("{{{index_col}}}")), &col);
        }
        generate_console(&content);
    }
    return;
}

fn process_templete(row: Vec<String>, template: &String) -> String {
    let mut output: String = template.to_string();
    for (index_col, col) in row.into_iter().enumerate() {
        let from = format!("{{{}}}", format!("{{{index_col}}}"));
        output = output.replace(&from, &col);
    }
    output
}

fn generate_list_image(list_data: Vec<Vec<String>>, from_opt: &FromArg) {
    let _ = create_dir_all(from_opt.common_arg.outdir.to_string())
        .expect("Cannot create output directory!");

    for row in list_data.iter() {
        let content = process_templete(row.to_vec(), &from_opt.template_content);
        let raw_filename = process_templete(row.to_vec(), &from_opt.template_filename);
        let text_top = process_templete(row.to_vec(), &from_opt.template_text_top);
        let text_bottom = process_templete(row.to_vec(), &from_opt.template_text_bottom);

        // Save the image with a unique filename
        let filename = raw_filename.replace("/", "_");
        let mut path_output_file: String =
            format!("{}/{}.png", from_opt.common_arg.outdir, filename);

        let mut counter = 0;
        while fs::metadata(&path_output_file).is_ok() {
            counter += 1;
            path_output_file = format!(
                "{}/{}_{}.png",
                from_opt.common_arg.outdir, filename, counter
            );
        }

        let result = generate_image(
            content.to_string(),
            from_opt.common_arg.size,
            &text_top,
            &text_bottom,
            from_opt.common_arg.top_space,
            from_opt.common_arg.bottom_space,
            &from_opt.common_arg.top_text_pos,
            &from_opt.common_arg.bottom_text_pos,
            &from_opt.common_arg.font_path,
            from_opt.common_arg.font_size,
            from_opt.common_arg.no_reduce_text_size,
            from_opt.common_arg.add_text_line_space,
        );

        handler_generate_image_result(result, path_output_file)
    }
}

fn handler_generate_image_result(
    result: Result<ResultGenerateImage, Box<dyn Error>>,
    path: String,
) {
    match result {
        Ok(r) => {
            // Show messsage when the title size was reduced
            if r.reduce_top_text_size.is_some() {
                println!(
                    "Info: Reduce font size of top: {}% > {}",
                    r.reduce_top_text_size.unwrap(),
                    path
                );
            }

            // Show messsage when the title size was reduced
            if r.reduce_bottom_text_size.is_some() {
                println!(
                    "Info: Reduce font size of bottom: {}% > {}",
                    r.reduce_bottom_text_size.unwrap(),
                    path
                );
            }

            let save_image = r.image_buffer.save(&path);

            match save_image {
                Ok(_) => println!("created: {}", &path),
                Err(_) => println!("Error saving image"),
            }
        }
        Err(e) => println!("Error: {} > {}", e, path),
    }
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
    let result_loop_check_size = 'loop_check_size: loop {
        for (index_line, line) in text_split.clone().enumerate() {
            // Define the text to be drawn
            let text = line;

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

            height_texts_sum += _text_size.1;

            if _text_size.0 < base_size && height_texts_sum < additional_space {
                continue;
            } else {
                if no_reduce_text_size {
                    break 'loop_check_size Some("Text size is over base image. try run without flag `--nrts` no_reduce_text_size.");
                }

                per_font_size -= 1;
                height_texts_sum = 0;
                text_draw_data = Vec::new();
                continue 'loop_check_size;
            }
        }
        break 'loop_check_size None;
    };

    if result_loop_check_size.is_some() {
        return Err(result_loop_check_size.unwrap().to_owned());
    }

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

fn generate_image(
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
) -> Result<ResultGenerateImage, Box<dyn Error>> {
    // Generate a QR code and convert it to ImageBuffer
    let qr_code_buffer = qrcode_generator::to_image_buffer(content, QrCodeEcc::Low, size as usize)
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
        Rgba([255, 255, 255, 255]),
    );

    // Copy the QR code image onto the new image
    for x in 0..qr_code_buffer.width() {
        for y in 0..qr_code_buffer.height() {
            let pixel_value = qr_code_buffer.get_pixel(x, y)[0];
            new_image.put_pixel(
                x,
                y + cal_top_space as u32,
                Rgba([pixel_value, pixel_value, pixel_value, 255]),
            );
        }
    }

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

        let color = Rgba([0, 0, 0, 255]);

        if !text_bottom.is_empty() {
            let text_bottom_draw_data = prepare_text_draw(
                text_bottom.to_string(),
                size,
                font_size as u32,
                &font,
                add_text_line_space,
                cal_bottom_space as u32,
                no_reduce_text_size,
            )
            .expect("Error: Prepare text draw (bottom)");

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

    let output_image = new_image.clone();

    Ok(ResultGenerateImage {
        image_buffer: output_image,
        reduce_top_text_size,
        reduce_bottom_text_size,
    })
}

fn process_draw_text(
    text_draw_data: Vec<TextDraw>,
    height_texts_sum: u32,
    line_height: u32,
    base_image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    side_space: u32,
    color: Rgba<u8>,
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

// Print the given qrcode object to the console
fn print_qr(qr: &Vec<Vec<bool>>) {
    let symbol: char = '█';

    let print_border = || {
        for _ in 0..&qr.len() + 2 {
            print!("{0}{0}", symbol);
        }
    };

    print_border();
    println!();

    for y in qr {
        for (x_index, x) in y.iter().enumerate() {
            if x_index == 0 {
                print!("{0}{0}", symbol);
            }
            let c: char = if !*x { symbol } else { ' ' };
            print!("{0}{0}", c);
            if x_index + 1 == y.len() {
                print!("{0}{0}", symbol);
            }
        }
        println!();
    }

    print_border();
    println!();
}

// Get csv file
fn process_file(path: &String) -> Result<Vec<Vec<String>>, ErrorCsv> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    let mut csv_data: Vec<Vec<String>> = Vec::new();

    for record in reader.records() {
        csv_data.push(record.unwrap().iter().map(String::from).collect());
    }

    Ok(csv_data)
}
