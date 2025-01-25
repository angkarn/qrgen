use base64::{engine::general_purpose, Engine};
use clap::{CommandFactory, Parser, Subcommand};
use rayon::prelude::*;
use rust_text_draw::{
    fontdb::{self},
    image::ImageFormat,
};
use std::{
    collections::HashMap,
    fs::{create_dir_all, read},
    io::Cursor,
};

static FONT_DEFAULT: &'static [u8] = include_bytes!("../fonts/poppins-v21-latin-regular.ttf");

/// QR Code Generator Tools
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,

    /// Print help
    #[arg(long = "help", hide_short_help = true)] // Define only the `--help` flag
    help: bool,
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

    /// Size of image width
    #[clap(short = 'w', long = "image_width", default_value = "1000")]
    image_width: u32,

    /// Size of image height
    #[clap(short = 'h', long = "image_height", default_value = "1000")]
    image_height: u32,

    /// Size of qr
    #[clap(short = 'q', long = "qr_size", default_value = "1000")]
    qr_size: u32,

    /// Output directory
    #[clap(short = 'o', long, default_value = "output")]
    outdir: String,

    /// Start position qr x axis
    #[clap(short = 'x', long = "pos_x", default_value = "0")]
    pos_qr_x: u32,

    /// Start position qr y axis
    #[clap(short = 'y', long = "pos_y", default_value = "0")]
    pos_qr_y: u32,

    /// Template of text render (json5)
    #[clap(short = 'r', long = "ttr")]
    template_text_render: Option<String>,

    /// Paths of font file
    #[clap(long = "fp", value_delimiter = ',')]
    font_path: Option<Vec<String>>,

    /// Font size (percentage)
    #[clap(long = "fs", default_value = "50")]
    font_size: u32,

    /// minimum font size of reduce. 0 = no reduce (replace on template)
    #[clap(long = "rfs", default_value = "0", hide = true)]
    reduce_font_size: u32,

    /// The error correction level in a QR Code symbol. (l|m|q|h)
    #[clap(long = "ecc", default_value = "m")]
    error_correction_level: String,
}

#[derive(Parser, Debug)]
#[command(
    after_help = "Template can be use `{{Number of column}}` to replace data of column. And use `{{ROW}}` to replace number of row. "
)]
struct FromArg {
    /// Path file of list content
    path: String,

    /// Template of qr content
    #[clap(short = 'c', long = "tc", default_value = "{{1}}")]
    template_content: String,

    /// Template filename.
    #[clap(short = 'n', long = "tfn", default_value = "{{1}}")]
    template_filename: String,

    #[command(flatten)]
    common_arg: CommonArg,
}

#[derive(Parser, Debug)]
struct GenArg {
    /// Content to generate qrcode
    content: String,

    #[command(flatten)]
    common_arg: CommonArg,
}

fn main() {
    use std::time::Instant;
    let now = Instant::now();

    let args = Args::parse();

    // println!("{:?}", args);

    if args.help {
        let mut cmd = Args::command();
        cmd.print_help().unwrap();
        println!(); // Add a newline for proper formatting
        return;
    }

    match &args.command {
        Command::Gen(state) => handle_gen_command(state),
        Command::From(state) => handle_from_command(state),
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn handle_gen_command(gen_opt: &GenArg) {
    // if gen_opt.common_arg.template_text_render {
    let font_db = get_font_db(gen_opt.common_arg.font_path.clone()); //.into_locale_and_db();

    let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
        image_width: gen_opt.common_arg.image_width,
        image_height: gen_opt.common_arg.image_height,
        qr_size: gen_opt.common_arg.qr_size,
        pos_qr_x: gen_opt.common_arg.pos_qr_x,
        pos_qr_y: gen_opt.common_arg.pos_qr_y,
        error_correction_level: gen_opt.common_arg.error_correction_level.clone(),
        template_text_render: gen_opt.common_arg.template_text_render.clone(),
        font_size: gen_opt.common_arg.font_size,
        reduce_font_size: gen_opt.common_arg.reduce_font_size,
        font_db,
    };

    match gen_opt.common_arg.format.as_str() {
        "console" => qrgen::utils::console::print_qr(&gen_opt.content),
        "png" => {
            let _ = create_dir_all(&gen_opt.common_arg.outdir.to_string())
                .expect("Cannot create output directory!");

            println!("Generate Image...");

            let result =
                qrgen::utils::generate::generate_image(gen_opt.content.clone(), gen_image_opt);

            handler_result_generate_image(
                1,
                &result,
                format!("{}/{}.png", gen_opt.common_arg.outdir, "qr"),
                false,
            );
        }
        "base64" => {
            let result =
                qrgen::utils::generate::generate_image(gen_opt.content.clone(), gen_image_opt);

            handler_result_generate_image(1, &result, "".to_owned(), true);
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

fn get_font_db(fonts_path: Option<Vec<String>>) -> fontdb::Database {
    let mut font_db = fontdb::Database::new();

    if fonts_path.is_some() {
        for path in &fonts_path.unwrap() {
            let font_data = read(path).expect(&format!("Error read font file: \"{}\"", path));
            font_db.load_font_data(font_data);
        }
    }

    font_db.load_font_data(FONT_DEFAULT.to_vec());
    font_db
}

fn generate_list_image(list_data: Vec<Vec<String>>, from_opt: &FromArg, to_base64: bool) {
    create_dir_all(from_opt.common_arg.outdir.to_string())
        .expect("Cannot create output directory!");

    let font_db = get_font_db(from_opt.common_arg.font_path.clone()); //.into_locale_and_db();

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
            format!("{}/{}.png", &from_opt.common_arg.outdir, filename)
        })
        .collect();

    // Process generate qr image
    let result_generate_image: Vec<bool> = list_data
        .par_iter()
        .enumerate()
        .map(|(index, row)| -> bool {
            let content =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_content, index);

            let template_text_render: Option<String> =
                match &from_opt.common_arg.template_text_render {
                    Some(t) => Some(qrgen::utils::template::from_vec(row.to_vec(), &t, index)),
                    None => None,
                };

            let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
                image_width: from_opt.common_arg.image_width,
                image_height: from_opt.common_arg.image_height,
                qr_size: from_opt.common_arg.qr_size,
                pos_qr_x: from_opt.common_arg.pos_qr_x,
                pos_qr_y: from_opt.common_arg.pos_qr_y,
                error_correction_level: from_opt.common_arg.error_correction_level.clone(),
                template_text_render: template_text_render,
                font_size: from_opt.common_arg.font_size,
                reduce_font_size: from_opt.common_arg.reduce_font_size,
                font_db: font_db.clone(),
            };

            let generate_image_result =
                qrgen::utils::generate::generate_image(content.to_string(), gen_image_opt);

            handler_result_generate_image(
                index + 1,
                &generate_image_result,
                list_data_file_name.get(index).unwrap().to_string(),
                to_base64,
            )
        })
        .collect();

    let count_success = result_generate_image.iter().filter(|x| **x).count();
    let count_error = result_generate_image.iter().count() - count_success;

    println!("Success {}, Error {} files.", count_success, count_error);
}

fn handler_result_generate_image(
    row_number: usize,
    result: &Result<qrgen::utils::generate::ResultGenerateImage, String>,
    path: String,
    to_base64: bool,
) -> bool {
    match result {
        Err(e) => {
            println!("Error: row: {} > {:#?}", row_number, e);
            false
        }
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

                println!("result_base64:{}:{}", path, &b64);
                true
            } else {
                // luma8
                // let save_image = r.image_buffer.into_luma8().save(&path);
                let save_image = r.image_buffer.save(&path);

                match save_image {
                    Ok(_) => {
                        println!("Created: {:?}", &path);
                        true
                    }
                    Err(e) => {
                        println!("Error: {} > {}", e, path);
                        false
                    }
                }
            }
        }
    }
}
