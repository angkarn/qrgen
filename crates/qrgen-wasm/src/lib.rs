//! qrgen for the browser.
//!
//! ```js
//! import init, { Batch } from "./qrgen.js";
//! await init();
//! const files = new Map([["data.csv", csvBytes], ["fonts/body.ttf", fontBytes]]);
//! const batch = new Batch(jobJson5, files);
//! for (let i = 0; i < batch.length; i++) {
//!   const image = batch.render(i); // { name, png, warnings }
//! }
//! ```
//!
//! A `Batch` renders on the thread that made it. To render in parallel, make one per Web Worker
//! and split the rows between them; `batch.files` lists the files the others need.

use std::{cell::RefCell, collections::BTreeSet};

use js_sys::{Map, Reflect, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};

/// The page's files: a `Map` or a plain object of path → `Uint8Array`, read only when needed.
struct Files {
    files: JsValue,
    read: RefCell<BTreeSet<String>>,
}

impl qrgen::FileSource for Files {
    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        let path = path.strip_prefix("./").unwrap_or(path);
        let value = match self.files.dyn_ref::<Map>() {
            Some(map) => map.get(&path.into()),
            None => Reflect::get(&self.files, &path.into()).unwrap_or(JsValue::UNDEFINED),
        };
        if value.is_undefined() {
            return Err("no such file".into());
        }
        let bytes = value
            .dyn_into::<Uint8Array>()
            .map_err(|_| "not a Uint8Array".to_string())?;
        self.read.borrow_mut().insert(path.to_string());
        Ok(bytes.to_vec())
    }
}

/// A job loaded and ready to render, one image per row.
#[wasm_bindgen]
pub struct Batch {
    inner: qrgen::Batch,
    single: bool,
    dir: String,
    files: Vec<String>,
}

#[wasm_bindgen]
impl Batch {
    /// Loads a JSON5 job with the files it refers to (`Map` or object of path → `Uint8Array`).
    #[wasm_bindgen(constructor)]
    pub fn new(job: &str, files: JsValue) -> Result<Batch, JsError> {
        let job = qrgen::Job::from_json5(job).map_err(to_js)?;
        if !files.is_object() {
            return Err(JsError::new("files must be a Map or an object"));
        }
        let files = Files {
            files,
            read: RefCell::default(),
        };
        let inner = qrgen::Batch::new(&job, &files).map_err(to_js)?;
        Ok(Self {
            inner,
            single: job.is_single(),
            dir: job.output.dir.unwrap_or_else(|| "output".into()),
            files: files.read.into_inner().into_iter().collect(),
        })
    }

    /// Number of images: CSV rows, or 1.
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.inner.len()
    }

    /// `true` when the job makes one image (no CSV).
    #[wasm_bindgen(getter)]
    pub fn single(&self) -> bool {
        self.single
    }

    /// Output directory for CSV rows (`output.dir`, default `output`).
    #[wasm_bindgen(getter, js_name = outputDir)]
    pub fn output_dir(&self) -> String {
        self.dir.clone()
    }

    /// Paths of the files the job read: pass only these to other workers rendering the same job.
    #[wasm_bindgen(getter)]
    pub fn files(&self) -> Vec<String> {
        self.files.clone()
    }

    /// File name of row `i` (e.g. `card_1.png`).
    pub fn name(&self, i: usize) -> Result<String, JsError> {
        self.check(i)?;
        self.inner
            .name(i)
            .map(str::to_string)
            .map_err(|e| JsError::new(&e))
    }

    /// Renders row `i`.
    pub fn render(&self, i: usize) -> Result<Image, JsError> {
        self.check(i)?;
        let name = self.name(i)?;
        let output = self.inner.render(i).map_err(to_js)?;
        let png = qrgen::encode_png(&output.image).map_err(to_js)?;
        let warnings = output
            .report
            .overflows
            .iter()
            .map(ToString::to_string)
            .collect();
        Ok(Image {
            name,
            png,
            warnings,
        })
    }
}

impl Batch {
    fn check(&self, i: usize) -> Result<(), JsError> {
        if i < self.inner.len() {
            Ok(())
        } else {
            Err(JsError::new(&format!(
                "row index {i} out of range (0..{})",
                self.inner.len()
            )))
        }
    }
}

/// One rendered image.
#[wasm_bindgen]
pub struct Image {
    name: String,
    png: Vec<u8>,
    warnings: Vec<String>,
}

#[wasm_bindgen]
impl Image {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// PNG file bytes.
    #[wasm_bindgen(getter)]
    pub fn png(&self) -> Uint8Array {
        Uint8Array::from(self.png.as_slice())
    }

    /// Text that did not fit its box (an error instead with `strict: true`).
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

/// qrgen version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

fn to_js(e: qrgen::Error) -> JsError {
    JsError::new(&e.to_string())
}
