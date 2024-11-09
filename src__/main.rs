use base64::{engine::general_purpose, Engine};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use rust_text_render::{
    fontdb::{self},
    image::ImageFormat,
    FontSystem,
};
use std::{
    collections::HashMap,
    fs::{create_dir_all, read},
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

    /// Size of qr
    #[clap(short = 's', long, default_value = "1000")]
    qr_size: u32,

    /// Output directory
    #[clap(short = 'o', long, default_value = "output")]
    outdir: String,

    /// Size of left space
    #[clap(long = "ls", default_value = "0")]
    left_space: u32,

    /// Size of top space
    #[clap(long = "ts", default_value = "0")]
    top_space: u32,

    /// Size of right space
    #[clap(long = "rs", default_value = "0")]
    right_space: u32,

    /// Size of bottom space
    #[clap(long = "bs", default_value = "0")]
    bottom_space: u32,

    /// text render template
    #[clap(long = "trt")]
    text_render_template: Option<String>,

    /// Paths of font file
    #[clap(long = "fp", value_delimiter = ',')]
    font_path: Option<Vec<String>>,

    /// Font size (percentage)
    #[clap(long = "fs", default_value = "10")]
    font_size: u32,

    /// text line height (multiply font size)
    #[clap(long = "tlh", default_value = "1.2")]
    text_line_height: f32,

    /// minimum font size of reduce. 0 = no reduce (replace on template)
    #[clap(long = "rfs", default_value = "0", hide = true)]
    reduce_font_size: u32,

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
    use std::time::Instant;
    let now = Instant::now();

    let args = Args::parse();

    // println!("{:?}", args);

    match &args.command {
        Command::Gen(state) => handle_gen_command(state),
        Command::From(state) => handle_from_command(state),
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn handle_gen_command(gen_opt: &GenArg) {
    // if gen_opt.common_arg.text_render_template {
    let locale_and_db =
        get_font_system(gen_opt.common_arg.font_path.clone().unwrap()).into_locale_and_db();

    let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
        qr_size: gen_opt.common_arg.qr_size,
        error_correction_level: gen_opt.common_arg.error_correction_level.clone(),
        left_space: gen_opt.common_arg.left_space,
        top_space: gen_opt.common_arg.top_space,
        right_space: gen_opt.common_arg.right_space,
        bottom_space: gen_opt.common_arg.bottom_space,
        text_render_template: gen_opt.common_arg.text_render_template.clone(),
        font_size: gen_opt.common_arg.font_size,
        text_line_height: gen_opt.common_arg.text_line_height,
        reduce_font_size: gen_opt.common_arg.reduce_font_size,
        locale_and_db: locale_and_db,
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
                &result,
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

            match handler_result_generate_image(&result, "".to_owned(), true) {
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
    for (index, row) in list_data.iter().enumerate() {
        let content =
            qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_content, index);
        qrgen::utils::console::print_qr(&content)
    }
    return;
}

fn get_font_system(fonts_path: Vec<String>) -> FontSystem {
    let font_db = fontdb::Database::new();
    let mut font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), font_db);

    for path in &fonts_path {
        let font_data = read(path).expect(&format!("Error read font file: \"{}\"", path));
        font_system.db_mut().load_font_data(font_data);
    }

    font_system
}

fn generate_list_image(list_data: Vec<Vec<String>>, from_opt: &FromArg, to_base64: bool) {
    let _ = create_dir_all(from_opt.common_arg.outdir.to_string())
        .expect("Cannot create output directory!");

    let locale_and_db =
        get_font_system(from_opt.common_arg.font_path.clone().unwrap()).into_locale_and_db();

    // Generate file name list
    let mut file_name_count_map: HashMap<String, u32> = HashMap::new();
    let list_data_file_name: Vec<String> = list_data
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let raw_filename =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_filename, index);
            let mut filename = raw_filename.replace("/", "_");
            let number_dup = file_name_count_map.get(&filename).unwrap_or(&0).clone();
            file_name_count_map.insert(filename.clone(), number_dup + 1);
            if number_dup > 0 {
                filename = format!("{}_{}", filename, number_dup + 1);
            }
            filename
        })
        .collect();

    // Process generate qr image
    let result_generate_image: Vec<bool> = list_data
        .par_iter()
        .enumerate()
        .map(|(index, row)| {
            let content =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_content, index);

            let text_render_template: Option<String> =
                match &from_opt.common_arg.text_render_template {
                    Some(t) => Some(qrgen::utils::template::from_vec(row.to_vec(), &t, index)),
                    None => None,
                };

            // Save the image with a unique filename
            let path_output_file: String = format!(
                "{}/{}.png",
                from_opt.common_arg.outdir,
                list_data_file_name.get(index).unwrap_or(&index.to_string())
            );

            let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
                qr_size: from_opt.common_arg.qr_size,
                error_correction_level: from_opt.common_arg.error_correction_level.clone(),
                left_space: from_opt.common_arg.left_space,
                top_space: from_opt.common_arg.top_space,
                right_space: from_opt.common_arg.right_space,
                bottom_space: from_opt.common_arg.bottom_space,
                text_render_template: text_render_template,
                font_size: from_opt.common_arg.font_size,
                text_line_height: from_opt.common_arg.text_line_height,
                reduce_font_size: from_opt.common_arg.reduce_font_size,
                locale_and_db: locale_and_db.clone(),
            };

            let result = qrgen::utils::generate::generate_image(content.to_string(), gen_image_opt);

            match handler_result_generate_image(&result, path_output_file, to_base64) {
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
    result: &Result<qrgen::utils::generate::ResultGenerateImage, String>,
    path: String,
    to_base64: bool,
) -> Result<String, String> {
    match result {
        Ok(r) => {
            // Show messsage when font size was reduced
            if r.reduce_font_size {
                println!("Info: Reduce font size of: {}", path);
            }

            // Show messsage when some draw out pixel
            if r.draw_out_pixel {
                println!("Info: some draw of pixel: {}", path);
            }

            if to_base64 {
                let mut bytes: Vec<u8> = Vec::new();
                r.image_buffer
                    .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
                    .expect("Couldn't write image to bytes.");

                let b64 = general_purpose::STANDARD.encode(bytes);
                Ok(format!("result_base64:{}:{}", path, &b64))
            } else {
                // luma8
                // let save_image = r.image_buffer.into_luma8().save(&save_path);

                let save_image = r.image_buffer.save(&path);

                match save_image {
                    Ok(_) => Ok(format!("Created: {:?}", &path)),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
        Err(e) => Err(format!("Error: {} > {}", e, path)),
    }
}
