use std::{fs, path::Path, process::Command};

use base64::{engine::general_purpose::STANDARD, Engine};

fn qrgen(dir: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_qrgen"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into(),
        String::from_utf8_lossy(&out.stderr).into(),
    )
}

fn decode_png(bytes: &[u8]) -> Vec<String> {
    let gray = image::load_from_memory(bytes).unwrap().into_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(gray);
    prepared
        .detect_grids()
        .iter()
        .map(|g| g.decode().unwrap().1)
        .collect()
}

#[test]
fn gen_writes_a_scannable_png() {
    let dir = tempfile::tempdir().unwrap();
    let (ok, stdout, stderr) = qrgen(
        dir.path(),
        &[
            "gen",
            "https://example.com/?q=ทดสอบ",
            "--width",
            "300",
            "-o",
            "out/qr.png",
        ],
    );
    assert!(ok, "{stderr}");
    assert_eq!(stdout.trim(), "out/qr.png", "prints the path it wrote");
    let png = fs::read(dir.path().join("out/qr.png")).unwrap();
    assert_eq!(decode_png(&png), ["https://example.com/?q=ทดสอบ"]);
}

#[test]
fn gen_base64_to_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let (ok, stdout, _) = qrgen(
        dir.path(),
        &["gen", "hello", "--width", "200", "--format", "base64"],
    );
    assert!(ok);
    assert_eq!(
        decode_png(&STANDARD.decode(stdout.trim()).unwrap()),
        ["hello"]
    );
}

#[test]
fn batch_with_headers_names_and_bad_rows() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("people.csv"),
        "name,id\nO'Brien,1\nAnn,2\nAnn,3\nNoId\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("card.json5"),
        r#"[{ type: "qr", height: "80%" }, { type: "text", y: "80%", text: "{{name}} \"{{id}}\"", size: 24 }]"#,
    )
    .unwrap();
    let (ok, stdout, stderr) = qrgen(
        dir.path(),
        &[
            "batch",
            "people.csv",
            "--header",
            "-t",
            "card.json5",
            "--width",
            "300",
            "--data",
            "id:{{id}}",
            "--name",
            "{{name}}",
        ],
    );
    assert!(!ok, "a failed row makes the exit code non-zero");
    assert!(
        stderr.contains("row 4: error: missing variable \"id\""),
        "{stderr}"
    );
    assert!(stderr.contains("3 created, 1 failed"), "{stderr}");
    let mut created: Vec<_> = stdout
        .lines()
        .map(|l| {
            Path::new(l)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    created.sort();
    assert_eq!(created, ["Ann.png", "Ann_2.png", "O'Brien.png"]);
    let png = fs::read(dir.path().join("output/O'Brien.png")).unwrap();
    assert_eq!(decode_png(&png), ["id:1"]);
}

#[test]
fn run_resolves_paths_from_the_job_file() {
    let dir = tempfile::tempdir().unwrap();
    let job_dir = dir.path().join("jobs");
    fs::create_dir_all(job_dir.join("img")).unwrap();
    image::RgbaImage::from_pixel(240, 160, image::Rgba([20, 80, 160, 255]))
        .save(job_dir.join("img/bg.png"))
        .unwrap();
    fs::write(job_dir.join("rows.csv"), "a,bg.png\nb,missing.png\n").unwrap();
    fs::write(
        job_dir.join("job.json5"),
        r#"{
          csv: "rows.csv",
          canvas: { base_image: "img/{{2}}" },
          template: [{ type: "qr", x: "50%", width: "50%" }],
          output: { dir: "out" },
        }"#,
    )
    .unwrap();
    let (ok, _, stderr) = qrgen(dir.path(), &["run", "jobs/job.json5"]);
    assert!(!ok);
    assert!(
        stderr.contains("row 2: error: cannot read image"),
        "{stderr}"
    );
    let png = fs::read(job_dir.join("out/1.png")).unwrap();
    let img = image::load_from_memory(&png).unwrap();
    assert_eq!(
        (img.width(), img.height()),
        (240, 160),
        "canvas takes the base image size"
    );
    assert_eq!(decode_png(&png), ["a"]);
}

#[test]
fn invalid_input_is_an_error_not_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("t.json5"),
        r#"[{ type: "text", txt: "x" }]"#,
    )
    .unwrap();
    let (ok, _, stderr) = qrgen(dir.path(), &["gen", "x", "-t", "t.json5"]);
    assert!(!ok);
    assert!(
        stderr.contains("unknown field `txt`") && stderr.contains("nodes[0]"),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked"));

    let (ok, _, stderr) = qrgen(dir.path(), &["gen", "x", "--dark", "nope"]);
    assert!(!ok);
    assert!(stderr.contains("invalid color"), "{stderr}");
}

#[test]
fn show_prints_blocks() {
    let (ok, stdout, _) = qrgen(Path::new("."), &["show", "hi"]);
    assert!(ok);
    assert!(stdout.lines().count() > 10 && stdout.contains('█'));
}
