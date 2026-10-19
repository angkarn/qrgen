use qrgen::{default_template, Canvas, Ecc, Error, Generator, QrStyle, Row};
use textplate::{
    image::{DynamicImage, Rgba, RgbaImage},
    Assets, Color, Fonts, Rect, Template, TemplateString,
};

fn fonts() -> Fonts {
    let mut fonts = Fonts::new();
    fonts.add("default", qrgen::DEFAULT_FONT).unwrap();
    fonts
}

/// Decodes every QR code found in the image.
fn decode(img: &RgbaImage) -> Vec<String> {
    let gray = DynamicImage::ImageRgba8(img.clone()).into_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(gray);
    prepared
        .detect_grids()
        .iter()
        .map(|g| g.decode().unwrap().1)
        .collect()
}

fn row_with_data<'a>(columns: &'a [String], data: &str) -> Row<'a> {
    Row::new(0, columns)
        .with_data(&TemplateString::parse(data))
        .unwrap()
}

#[test]
fn default_template_scans() {
    let generator = Generator::new(
        fonts(),
        default_template(),
        Canvas {
            width: 300,
            ..Canvas::default()
        },
        QrStyle::default(),
    );
    let columns = vec!["https://example.com/ข้อมูล?id=1".to_string()];
    let out = generator
        .render(&row_with_data(&columns, "{{1}}"), &Assets::new())
        .unwrap();
    assert_eq!(out.image.dimensions(), (300, 300));
    assert_eq!(decode(&out.image), [columns[0].clone()]);
}

#[test]
fn qr_card_with_text() {
    let template = Template::from_json5(
        r##"[
          { type: "qr", height: "80%", ecc: "h", dark: "#1f2937" },
          { type: "text", y: "80%", height: "20%", text: "No. {{ROW}} {{2}}", size: "6vw", align: "center", valign: "middle" },
        ]"##,
    )
    .unwrap();
    let canvas = Canvas {
        width: 400,
        height: Some(500),
        ..Canvas::default()
    };
    let generator = Generator::new(fonts(), template, canvas, QrStyle::default());
    let columns = vec!["A-001".to_string(), "Ann".to_string()];
    let out = generator
        .render(&row_with_data(&columns, "{{1}}"), &Assets::new())
        .unwrap();
    assert_eq!(decode(&out.image), ["A-001"]);
    assert!(out.report.overflows.is_empty());
    // caption drawn below the QR
    let caption_ink = (0..400)
        .flat_map(|x| (420..480).map(move |y| (x, y)))
        .filter(|&(x, y)| out.image.get_pixel(x, y).0[0] < 128)
        .count();
    assert!(caption_ink > 100, "caption pixels: {caption_ink}");
}

#[test]
fn transparent_colors_on_base_image() {
    let mut assets = Assets::new();
    assets.insert(
        "bg-blue.png",
        RgbaImage::from_pixel(240, 240, Rgba([30, 60, 200, 255])),
    );
    let canvas = Canvas {
        base_image: Some(TemplateString::parse("bg-{{2}}.png")),
        ..Canvas::default()
    };
    let style = QrStyle {
        dark: Color::rgba(0, 0, 0, 255),
        light: Color::rgba(255, 255, 255, 200),
        ..QrStyle::default()
    };
    let generator = Generator::new(fonts(), default_template(), canvas, style);
    let columns = vec!["hello".to_string(), "blue".to_string()];
    let row = row_with_data(&columns, "{{1}}");

    assert_eq!(generator.asset_names(&row).unwrap(), ["bg-blue.png"]);
    let out = generator.render(&row, &assets).unwrap();
    assert_eq!(
        out.image.dimensions(),
        (240, 240),
        "canvas takes the base image size"
    );
    assert_eq!(decode(&out.image), ["hello"]);
    let corner = out.image.get_pixel(1, 1).0;
    assert!(
        corner[2] > corner[0],
        "light modules let the blue base show through: {corner:?}"
    );

    let other = vec!["hello".to_string(), "red".to_string()];
    let err = generator
        .render(&row_with_data(&other, "{{1}}"), &assets)
        .err()
        .unwrap();
    assert!(
        matches!(err, Error::MissingBaseImage(ref n) if n == "bg-red.png"),
        "{err}"
    );
}

#[test]
fn qr_node_errors_are_reported() {
    let render = |src: &str, data: &str| {
        let generator = Generator::new(
            fonts(),
            Template::from_json5(src).unwrap(),
            Canvas {
                width: 100,
                ..Canvas::default()
            },
            QrStyle::default(),
        );
        let columns = vec![data.to_string()];
        generator
            .render(&row_with_data(&columns, "{{1}}"), &Assets::new())
            .err()
            .map(|e| e.to_string())
    };
    assert!(render(r#"[{ type: "qr", width: 10 }]"#, "x")
        .unwrap()
        .contains("QR needs at least"));
    assert!(render(r#"[{ type: "qr", ecc: "z" }]"#, "x")
        .unwrap()
        .contains("error correction"));
    assert!(render(r##"[{ type: "qr", colour: "#000" }]"##, "x")
        .unwrap()
        .contains("unknown field `colour`"));
    assert!(render(r#"[{ type: "qr" }]"#, &"x".repeat(4000))
        .unwrap()
        .contains("too long"));
}

#[test]
fn draw_qr_is_centered_and_crisp() {
    let mut img = RgbaImage::from_pixel(250, 200, Rgba([255, 255, 255, 255]));
    let style = QrStyle {
        ecc: Ecc::Low,
        quiet_zone: 0,
        ..QrStyle::default()
    };
    qrgen::draw_qr(&mut img, Rect::new(0.0, 0.0, 250.0, 200.0), "hi", &style).unwrap();
    // version 1: 21 modules -> 9px modules -> 189px, centered in 250x200
    let dark: Vec<_> = img
        .enumerate_pixels()
        .filter(|(_, _, p)| p.0[0] == 0)
        .map(|(x, y, _)| (x, y))
        .collect();
    let (x0, y0) = (
        dark.iter().map(|p| p.0).min().unwrap(),
        dark.iter().map(|p| p.1).min().unwrap(),
    );
    let (x1, y1) = (
        dark.iter().map(|p| p.0).max().unwrap(),
        dark.iter().map(|p| p.1).max().unwrap(),
    );
    assert_eq!((x1 - x0 + 1, y1 - y0 + 1), (189, 189));
    assert_eq!((x0, y0), (30, 5));
    assert!(
        img.pixels().all(|p| p.0[0] == 0 || p.0[0] == 255),
        "no anti-aliased module edges"
    );
}

#[test]
fn generator_is_shareable_across_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Generator>();
}
