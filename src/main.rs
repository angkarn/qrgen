use ab_glyph::{FontVec, PxScale};
use clap::{Parser, Subcommand};
use csv::Error;
use image::ImageBuffer;
use imageproc::image::Rgba;
use imageproc::{
    drawing::{draw_text_mut, text_size},
    image,
};
use qrcode_generator::QrCodeEcc;
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
    #[clap(short, long, default_value = "console")]
    format: String,

    /// Size of image
    #[clap(short = 's', long, default_value = "1024")]
    size: u32,

    /// Output directory
    #[clap(short, long, default_value = "output")]
    outdir: String,

    /// Side of additional space <top|bottom>
    #[clap(short, long = "ass", default_value = "bottom")]
    add_side_space: String,

    /// Size of additional space (percent of image size)
    #[clap(long = "ss", default_value = "15")]
    size_space: usize,

    /// Path of font file
    #[clap(long = "fp")]
    font_path: Option<String>,

    /// Font size (percentage)
    #[clap(long = "fs", default_value = "10")]
    font_size: usize,

    /// Positional of additional title (percentage), Empty this will center of additional space
    #[clap(long = "tpxy", value_delimiter = ',')]
    title_pos_xy: Vec<usize>,
}

#[derive(Parser, Debug)]
struct FromArg {
    /// Path file of list content
    path: String,

    /// Index of column for qr content
    #[clap(short, long = "icc", default_value = "0")]
    index_column_content: usize,

    /// Index of column for additional title
    #[clap(long = "ict")]
    index_column_title: Option<usize>,

    /// Template content to replace in list. use `,` for each column eg. `hello {{}}!,,col-3-{{}}`
    #[clap(short, long, default_value = "{{}}", value_delimiter = ',')]
    template: Vec<String>,

    /// Sumbol or substring for mark to replace on templete
    #[clap(long = "sr", default_value = "{{}}")]
    symbol_mark_replace: String,

    /// Index of column to set each file name
    #[clap(long = "icfn", default_value = "0")]
    index_column_filename: usize,

    #[command(flatten)]
    common_arg: CommonArg,
}

#[derive(Parser, Debug)]
struct GenArg {
    /// Content to generate qrcode
    content: String,

    /// Put additional title on image
    #[clap(short, long)]
    title: Option<String>,

    #[command(flatten)]
    common_arg: CommonArg,
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);

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

            generate_image(
                gen_opt.content.clone(),
                gen_opt.common_arg.size,
                &format!("{}/{}.png", gen_opt.common_arg.outdir, "qr"),
                &gen_opt.title,
                &gen_opt.common_arg.add_side_space,
                &gen_opt.common_arg.size_space,
                &gen_opt.common_arg.title_pos_xy,
                &gen_opt.common_arg.font_path,
                gen_opt.common_arg.font_size,
            )
        }
        _ => {}
    }
}

fn handle_from_command(from_opt: &FromArg) {
    let list_content = process_file(
        &from_opt.path,
        &from_opt.template,
        &from_opt.symbol_mark_replace,
    )
    .expect("Error processing file");

    println!("\nSample data processed:");
    print!(
        "> {:?}",
        list_content.get(0).expect("No data after process")
    );

    println!("\n\nGenerate Images...");

    match from_opt.common_arg.format.as_str() {
        "console" => generate_list_console(list_content, from_opt),

        "png" => generate_list_image(list_content, from_opt),
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
        let content = &row[from_opt.index_column_content];
        generate_console(content);
    }
    return;
}

fn generate_list_image(list_content: Vec<Vec<String>>, from_opt: &FromArg) {
    // // Check config file name is duplicate of other
    // for content in list_content.iter() {
    //     let found_record = list_content.iter().find(|s| {
    //         s[from_opt.index_column_filename as usize]
    //             == content[from_opt.index_column_filename as usize]
    //     });
    //     if found_record.is_some() {
    //         print!("Some config file name is duplicate of other");
    //         return;
    //     }
    // }

    let _ = create_dir_all(from_opt.common_arg.outdir.to_string())
        .expect("Cannot create output directory!");

    for content in list_content.iter() {
        // Save the image with a unique filename
        let filename = content[from_opt.index_column_filename].replace("/", "_");
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

        let title = match from_opt.index_column_title {
            Some(index) => Some(content.get(index).unwrap().to_string()),
            None => None,
        };

        generate_image(
            content[from_opt.index_column_content].to_string(),
            from_opt.common_arg.size,
            &path_output_file,
            &title,
            &from_opt.common_arg.add_side_space,
            &from_opt.common_arg.size_space,
            &from_opt.common_arg.title_pos_xy,
            &from_opt.common_arg.font_path,
            from_opt.common_arg.font_size,
        );
    }
}

fn generate_image(
    content: String,
    size: u32,
    path_output_file: &String,
    title: &Option<String>,
    add_side_space: &str,
    size_space: &usize,
    title_pos_xy: &[usize],
    font_path: &Option<String>,
    font_size: usize,
) {
    // Generate a QR code and convert it to ImageBuffer
    let qr_code_buffer = qrcode_generator::to_image_buffer(content, QrCodeEcc::Low, size as usize)
        .expect("Failed to generate QR code");

    // Define the additional space to be added on top
    let additional_space = if !title.is_none() {
        size * size_space.to_owned() as u32 / 100
    } else {
        0
    };

    // Create a new image with additional space at the top
    let mut new_image = ImageBuffer::from_pixel(
        qr_code_buffer.width(),
        qr_code_buffer.height() + additional_space,
        Rgba([255, 255, 255, 255]),
    );

    // Copy the QR code image onto the new image
    for x in 0..qr_code_buffer.width() {
        for y in 0..qr_code_buffer.height() {
            let pixel_value = qr_code_buffer.get_pixel(x, y)[0];
            new_image.put_pixel(
                x,
                match add_side_space {
                    "top" => y + additional_space,
                    _ => y,
                },
                Rgba([pixel_value, pixel_value, pixel_value, 255]),
            );
        }
    }

    let new_image_width = new_image.width();
    let new_image_height = new_image.height();

    if !title.is_none() {
        // Get font data
        let font_data = if font_path.is_none() {
            Vec::<u8>::from(FONT_DEFAULT)
        } else {
            read(font_path.clone().unwrap()).expect("Error read font file")
        };

        let font = FontVec::try_from_vec(font_data).expect("Error constructing Font");

        let color = Rgba([0, 0, 0, 255]);

        let mut px_scale: PxScale;

        // Define the text to be drawn
        let text = title.clone().unwrap().to_string();

        let mut _text_size: (u32, u32);

        let mut percent_font_size = font_size;

        // loop check and resize if text over size of image
        loop {
            // Define text properties
            px_scale = PxScale::from((size * percent_font_size as u32 / 100) as f32);

            // Calculate the total text width
            _text_size = text_size(px_scale, &font, &text);

            if _text_size.0 <= new_image_width {
                break;
            }
            percent_font_size -= 1;
        }

        // Show messsage when the title size was reduced
        if percent_font_size < font_size {
            println!(
                "info: title ({}) was reduced font to {}%",
                text, percent_font_size
            )
        }

        // Calculate the x-coordinate to center the text
        let x_center = (new_image_width - _text_size.0) / 2;

        // Render text onto the image
        draw_text_mut(
            &mut new_image,
            color,
            if let [x, _] = title_pos_xy {
                (new_image_width * *x as u32 / 100) as i32
            } else {
                x_center as i32
            },
            if let [_, y] = title_pos_xy {
                (new_image_height * *y as u32 / 100) as i32
            } else {
                // Calculate to center y
                match add_side_space {
                    "top" => ((additional_space * 50 / 100) - _text_size.1 / 2) as i32,
                    _ => {
                        ((size - (size * 5 / 100) + (additional_space * 50 / 100))
                            - _text_size.1 / 2) as i32
                    }
                }
            },
            px_scale,
            &font,
            &text,
        );
    }

    new_image
        .save(path_output_file)
        .expect("Error saving image");

    println!("created: {}", path_output_file);
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

// Get csv file and process data
fn process_file(
    path: &String,
    template: &Vec<String>,
    symbol_mark_replace: &String,
) -> Result<Vec<Vec<String>>, Error> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    let mut csv_data: Vec<Vec<String>> = Vec::new();

    for record in reader.records() {
        csv_data.push(
            record
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    if template.len() > i && template[i] != "" {
                        template[i].replace(symbol_mark_replace, x)
                    } else {
                        x.to_string()
                    }
                })
                .collect(),
        );
    }

    Ok(csv_data)
}
