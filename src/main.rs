use base64::{engine::general_purpose, Engine};
use clap::{CommandFactory, Parser, Subcommand};
use rayon::prelude::*;
use rust_text_draw::{
    fontdb::{self},
    image::ImageFormat,
    Widget,
};
use serde_json::{from_value, Value};
use std::{
    collections::HashMap,
    fs::{create_dir_all, read},
    io::Cursor,
    path::Path,
};

static FONT_DEFAULT: &'static [u8] = include_bytes!("../fonts/poppins-v21-latin-regular.ttf");

/// QR Code Generator and Draws Tools
#[derive(Parser, Debug, serde::Deserialize)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,

    /// Print help
    #[arg(long = "help", hide_short_help = true)] // Only the `--help` flag
    help: bool,
}

#[derive(Subcommand, Debug, serde::Deserialize)]
enum Command {
    /// Generate one QR code
    Gen(GenArg),

    /// Generate multiple QR codes from a CSV file
    From(FromArg),

    /// Run command from config file
    Config(Config),
}

#[derive(Parser, Debug, serde::Deserialize)]
#[serde(default)]
struct CommonArg {
    /// Output format (console|png|base64)
    #[clap(short = 'f', long, default_value = "console")]
    format: String,

    /// Path to base image file. Overrides image width/height (also works with data template)
    #[clap(short = 'b', long = "base_image")]
    base_image: Option<String>,

    /// QR color (1, like black)
    #[clap(short = '1', long = "qr_color_1", default_value = "000000ff")]
    qr_color_1: String,

    /// QR color (0, like white)
    #[clap(short = '0', long = "qr_color_0", default_value = "ffffffff")]
    qr_color_0: String,

    /// Fill background color
    #[clap(long = "fill", default_value = "ffffffff")]
    fill_color: String,

    /// Image width (pixels)
    #[clap(short = 'w', long = "image_width", default_value = "1000")]
    image_width: u32,

    /// Image height (pixels) (default: image width)
    #[clap(short = 'h', long = "image_height")]
    image_height: Option<u32>,

    /// QR size (pixels) (default: image width)
    #[clap(short = 's', long = "qr_size")]
    qr_size: Option<u32>,

    /// QR X position (pixels)
    #[clap(short = 'x', long = "pos_x", default_value = "0")]
    pos_qr_x: u32,

    /// QR Y position (pixels)
    #[clap(short = 'y', long = "pos_y", default_value = "0")]
    pos_qr_y: u32,

    /// Draw template (json5) (ignored from clap)
    #[clap(skip)]
    #[serde(skip)]
    template_draw: Option<Vec<Widget>>,

    /// Draw template as string (json5)
    #[clap(short = 'd', long = "td")]
    template_draw_string: Option<String>,

    /// Font file paths
    #[clap(long = "fp", value_delimiter = ',')]
    font_path: Option<Vec<String>>,

    /// Default font size (percentage of image width)
    #[clap(long = "fs", default_value = "3")]
    font_size: f32,

    /// Minimum font size for reduction. 0 = no reduction (replace in template)
    #[clap(long = "rfs", default_value = "0", hide = true)]
    reduce_font_size: u32,

    /// Output directory
    #[clap(short = 'o', long, default_value = "output")]
    outdir: String,

    /// QR error correction level (l|m|q|h)
    #[clap(long = "ecc", default_value = "m")]
    error_correction_level: String,
}

impl Default for CommonArg {
    fn default() -> Self {
        Self {
            format: "console".to_string(),
            base_image: None,
            qr_color_1: "000000ff".to_string(),
            qr_color_0: "ffffffff".to_string(),
            fill_color: "ffffffff".to_string(),
            image_width: 1000,
            image_height: None,
            qr_size: None,
            pos_qr_x: 0,
            pos_qr_y: 0,
            template_draw: None,
            font_path: None,
            font_size: 3.0,
            reduce_font_size: 0,
            outdir: "output".to_string(),
            error_correction_level: "m".to_string(),
            template_draw_string: None,
        }
    }
}

#[derive(Parser, Debug, serde::Deserialize)]
#[command(
    after_help = "Template can use `{{Number of column}}` to replace column data, and `{{ROW}}` to replace row number."
)]
#[serde(default)]
struct FromArg {
    /// Path to CSV file
    path: String,

    /// QR content template
    #[clap(short = 'c', long = "tc", default_value = "{{1}}")]
    template_content: String,

    /// Filename template
    #[clap(short = 'n', long = "tfn", default_value = "{{1}}")]
    template_filename: String,

    #[command(flatten)]
    #[serde(default)]
    common_arg: CommonArg,
}

impl Default for FromArg {
    fn default() -> Self {
        Self {
            path: Default::default(),
            template_content: "{{1}}".to_string(),
            template_filename: "{{1}}".to_string(),
            common_arg: Default::default(),
        }
    }
}

#[derive(Parser, Debug, serde::Deserialize)]
struct GenArg {
    /// QR code content
    content: String,

    #[command(flatten)]
    #[serde(default)]
    common_arg: CommonArg,
}

#[derive(Parser, Debug, serde::Deserialize)]
struct Config {
    /// Path to the config file
    path: String,
}

fn main() {
    use std::time::Instant;
    let now = Instant::now();

    let args = Args::parse();

    if args.help {
        let mut cmd = Args::command();
        cmd.print_help().unwrap();
        println!();
        return;
    }

    match &args.command {
        Command::Config(config) => run_command_from_config_file(&config.path),
        Command::Gen(state) => handle_gen_command(state),
        Command::From(state) => handle_from_command(state),
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

// Function to execute command from config file
fn run_command_from_config_file<P: AsRef<Path>>(path: P) {
    let content = std::fs::read_to_string(path).expect("Cannot read config file");

    #[derive(Debug, serde::Deserialize)]
    struct ConfigFile {
        command: String,
    }
    let config: ConfigFile = json5::from_str(&content).expect("Invalid config file format");

    let mut state: Value = json5::from_str(&content).expect("Invalid command format");

    let state_value: Value =
        json5::from_str(&content).expect("Invalid JSON5 format in config file");

    // Convert template_draw array to template_draw_string if present
    if state_value["common_arg"]["template_draw"].is_array() {
        state["common_arg"]["template_draw_string"] = Some(
            json5::to_string(&state_value["common_arg"]["template_draw"].clone())
                .expect("Invalid template draw format"),
        )
        .into();
    }

    match config.command.to_lowercase().as_str() {
        "gen" => {
            let state: GenArg = from_value(state).expect("Invalid Gen command format");
            handle_gen_command(&state);
        }
        "from" => {
            let state: FromArg = from_value(state).expect("Invalid From command format");
            handle_from_command(&state);
        }
        _ => {
            eprintln!("Unsupported command in config file.");
        }
    }
}

fn handle_gen_command(gen_opt: &GenArg) {
    let font_db = get_font_db(gen_opt.common_arg.font_path.clone());

    let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
        qr_color: (
            gen_opt.common_arg.qr_color_0.clone(),
            gen_opt.common_arg.qr_color_1.clone(),
        ),
        base_image: gen_opt.common_arg.base_image.clone(),
        fill_color: gen_opt.common_arg.fill_color.clone(),
        image_width: gen_opt.common_arg.image_width,
        image_height: gen_opt
            .common_arg
            .image_height
            .unwrap_or(gen_opt.common_arg.image_width),
        qr_size: gen_opt
            .common_arg
            .qr_size
            .unwrap_or(gen_opt.common_arg.image_width),
        pos_qr_x: gen_opt.common_arg.pos_qr_x,
        pos_qr_y: gen_opt.common_arg.pos_qr_y,
        error_correction_level: gen_opt.common_arg.error_correction_level.clone(),
        template_draw: Some(
            json5::from_str(&gen_opt.common_arg.template_draw_string.as_ref().unwrap())
                .expect("Invalid template draw format"),
        ),
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
            eprintln!("Format not found!")
        }
    }
}

fn generate_list_console(list_data: Vec<Vec<String>>, from_opt: &FromArg) {
    for (index, row) in list_data.iter().enumerate() {
        let content =
            qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_content, index);
        qrgen::utils::console::print_qr(&content)
    }
}

fn get_font_db(fonts_path: Option<Vec<String>>) -> fontdb::Database {
    let mut font_db = fontdb::Database::new();

    // Load default font file
    font_db.load_font_data(FONT_DEFAULT.to_vec());

    if let Some(paths) = fonts_path {
        for path in &paths {
            let font_data = read(path).expect(&format!("Error reading font file: \"{}\"", path));
            font_db.load_font_data(font_data);
        }
    }

    font_db
}

fn generate_list_image(list_data: Vec<Vec<String>>, from_opt: &FromArg, to_base64: bool) {
    create_dir_all(from_opt.common_arg.outdir.to_string())
        .expect("Cannot create output directory!");

    let font_db = get_font_db(from_opt.common_arg.font_path.clone());

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

    // Generate QR images
    let result_generate_image: Vec<bool> = list_data
        .par_iter()
        .enumerate()
        .map(|(index, row)| -> bool {
            let content =
                qrgen::utils::template::from_vec(row.to_vec(), &from_opt.template_content, index);

            let template_draw_string = match &from_opt.common_arg.template_draw_string {
                Some(t) => Some(qrgen::utils::template::from_vec(row.to_vec(), &t, index)),
                None => None,
            };

            let template_draw = match template_draw_string {
                Some(t) => Some(json5::from_str(&t).expect("Invalid template draw format")),
                None => None,
            };

            let base_image = match &from_opt.common_arg.base_image {
                Some(v) => Some(qrgen::utils::template::from_vec(row.to_vec(), v, index)),
                None => None,
            };

            let gen_image_opt = qrgen::utils::generate::GenerateImageOptions {
                qr_color: (
                    from_opt.common_arg.qr_color_0.clone(),
                    from_opt.common_arg.qr_color_1.clone(),
                ),
                base_image: base_image,
                fill_color: from_opt.common_arg.fill_color.clone(),
                image_width: from_opt.common_arg.image_width,
                image_height: from_opt
                    .common_arg
                    .image_height
                    .unwrap_or(from_opt.common_arg.image_width),
                qr_size: from_opt
                    .common_arg
                    .qr_size
                    .unwrap_or(from_opt.common_arg.image_width),
                pos_qr_x: from_opt.common_arg.pos_qr_x,
                pos_qr_y: from_opt.common_arg.pos_qr_y,
                error_correction_level: from_opt.common_arg.error_correction_level.clone(),
                template_draw: template_draw,
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

    println!("Success: {}, Error: {} files.", count_success, count_error);
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
            // Info when font size was reduced
            if r.reduce_font_size {
                println!("Info: Font size reduced for: {}", path);
            }

            // Info when some draw out pixel
            if r.draw_out_pixel {
                println!("Info: Some pixels drawn out of bounds: {}", path);
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
