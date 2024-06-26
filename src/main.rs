use base64::{engine::general_purpose, Engine};
use clap::{Parser, Subcommand};
use imageproc::image::ImageFormat;
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, metadata},
    io::Cursor,
};

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
    /// Format output (console|png|base64)
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

    /// The error correction level in a QR Code symbol. (l|m|q|h)
    #[clap(long = "ecc", default_value = "m")]
    error_correction_level: String,
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

fn main() {
    let args = Args::parse();

    // println!("{:?}", args);

    match &args.command {
        Command::Gen(state) => handle_gen_command(state),
        Command::From(state) => handle_from_command(state),
    }
}

fn handle_gen_command(gen_opt: &GenArg) {
    let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
        size: gen_opt.common_arg.size,
        text_top: gen_opt.top_text.clone(),
        text_bottom: gen_opt.bottom_text.clone(),
        top_space: gen_opt.common_arg.top_space,
        bottom_space: gen_opt.common_arg.bottom_space,
        top_text_pos: gen_opt.common_arg.top_text_pos.clone(),
        bottom_text_pos: gen_opt.common_arg.bottom_text_pos.clone(),
        font_path: gen_opt.common_arg.font_path.clone(),
        font_size: gen_opt.common_arg.font_size,
        no_reduce_text_size: gen_opt.common_arg.no_reduce_text_size,
        add_text_line_space: gen_opt.common_arg.add_text_line_space,
        error_correction_level: gen_opt.common_arg.error_correction_level.clone(),
    };

    match gen_opt.common_arg.format.as_str() {
        "console" => qrgen::utils::console::print_qr(&gen_opt.content),
        "png" => {
            let _ = create_dir_all(&gen_opt.common_arg.outdir.to_string())
                .expect("Cannot create output directory!");

            println!("\n\nGenerate Image...");

            let result =
                qrgen::utils::generate::generate_image(gen_opt.content.clone(), gen_image_opt);

            match handler_result_generate_image(
                result,
                format!("{}/{}.png", gen_opt.common_arg.outdir, "qr"),
                false,
            ) {
                Ok(r) => println!("{}", r),
                Err(e) => println!("{}", e),
            }
        }
        "base64" => {
            let result =
                qrgen::utils::generate::generate_image(gen_opt.content.clone(), gen_image_opt);

            match handler_result_generate_image(result, "".to_owned(), true) {
                Ok(r) => println!("{}", r),
                Err(e) => println!("{}", e),
            }
        }
        _ => {}
    }
}

fn handle_from_command(from_opt: &FromArg) {
    let list_data =
        qrgen::utils::process_file::csv_to_vec(&from_opt.path).expect("Error processing file");

    println!("Generate Images...");

    match from_opt.common_arg.format.as_str() {
        "console" => generate_list_console(list_data, from_opt),

        "png" => generate_list_image(list_data, from_opt, false),
        "base64" => generate_list_image(list_data, from_opt, true),
        _ => {
            eprintln!("Can't found this format!")
        }
    }
}

fn generate_list_console(list_data: Vec<Vec<String>>, from_opt: &FromArg) {
    for row in list_data {
        let content = qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_content);
        qrgen::utils::console::print_qr(&content)
    }
    return;
}

fn generate_list_image(list_data: Vec<Vec<String>>, from_opt: &FromArg, to_base64: bool) {
    let _ = create_dir_all(from_opt.common_arg.outdir.to_string())
        .expect("Cannot create output directory!");

    let result_generate_image: Vec<bool> = list_data
        .par_iter()
        .map(|row| {
            let content =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_content);
            let raw_filename =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_filename);
            let text_top =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_text_top);
            let text_bottom =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_text_bottom);

            // Save the image with a unique filename
            let filename = raw_filename.replace("/", "_");
            let mut path_output_file: String =
                format!("{}/{}.png", from_opt.common_arg.outdir, filename);

            let mut counter = 0;
            while metadata(&path_output_file).is_ok() {
                counter += 1;
                path_output_file = format!(
                    "{}/{}_{}.png",
                    from_opt.common_arg.outdir, filename, counter
                );
            }

            let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
                size: from_opt.common_arg.size,
                text_top: text_top,
                text_bottom: text_bottom,
                top_space: from_opt.common_arg.top_space,
                bottom_space: from_opt.common_arg.bottom_space,
                top_text_pos: from_opt.common_arg.top_text_pos.clone(),
                bottom_text_pos: from_opt.common_arg.bottom_text_pos.clone(),
                font_path: from_opt.common_arg.font_path.clone(),
                font_size: from_opt.common_arg.font_size,
                no_reduce_text_size: from_opt.common_arg.no_reduce_text_size,
                add_text_line_space: from_opt.common_arg.add_text_line_space,
                error_correction_level: from_opt.common_arg.error_correction_level.clone(),
            };

            let result = qrgen::utils::generate::generate_image(content.to_string(), gen_image_opt);

            match handler_result_generate_image(result, path_output_file, to_base64) {
                Ok(r) => {
                    println!("{}", r);
                    true
                }
                Err(e) => {
                    println!("{}", e);
                    false
                }
            }
        })
        .collect();

    let count_success = result_generate_image.iter().filter(|x| **x).count();
    let count_error = result_generate_image.iter().count() - count_success;

    println!("Success {}, Error {} files.", count_success, count_error);
}

fn handler_result_generate_image(
    result: Result<qrgen::utils::generate::ResultGenerateImage, String>,
    path: String,
    to_base64: bool,
) -> Result<String, String> {
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

            if to_base64 {
                let mut bytes: Vec<u8> = Vec::new();
                r.image_buffer
                    .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
                    .expect("Couldn't write image to bytes.");

                let b64 = general_purpose::STANDARD.encode(bytes);
                Ok(format!("{}", &b64))
            } else {
                let save_image = r.image_buffer.into_luma8().save(&path);

                match save_image {
                    Ok(_) => Ok(format!("Created: {}", &path)),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
        Err(e) => Err(format!("Error: {} > {}", e, path)),
    }
}
