# Changelog

## Unreleased

- Images used by one row (e.g. a photo per row) are decoded when that row renders instead of all
  up front: 200 rows of 2000x1500 photos went from 9.5 s and 2.4 GB peak memory to 3.7 s and
  190 MB. Images shared by several rows are still decoded once.
- Browser package is built with WebAssembly SIMD: about 3.5x faster rendering (200 cards in the
  demo: 3.3 s to 1.35 s). Needs Chrome 91, Firefox 89, Safari 16.4 or newer.

## v0.4.0

- Browser package (`crates/qrgen-wasm`, wasm-bindgen): `new Batch(job, files)` runs the same JSON5
  jobs as `qrgen run` with files from the page, and renders rows to PNGs. Released as
  `qrgen-web.tar.gz`.
- Library: `Job`, `Batch` and `FileSource` (feature `decode`) load and plan a job, so the CLI and
  the browser share one job format.
- Job and file errors read `cannot read CSV x.csv: ...` instead of `invalid value: ...`.
- `qrgen --version` prints `qrgen` instead of `qrgen-cli`.

## v0.3.0

A rewrite on [textplate](https://github.com/angkarn/textplate). Commands, options, config files
and templates have changed; v0.2.x files do not work with this version.

### Commands
- `gen <DATA>`: one image (`-o -` writes the PNG to stdout).
- `batch <CSV>` (was `from`): one image per row, in parallel. Supports `--header` for `{{name}}`
  placeholders, `--fail-fast` and `--jobs`.
- `run <JOB>` (was `config`): job file with all options and an inline template. Relative paths
  are resolved from the job file's folder.
- `show <DATA>`: prints the QR code in the terminal with half-block characters.
- Options use kebab-case (`--base-image`, `--font NAME=PATH`, `--quiet-zone`, ...) and `-h` is help.

### Templates
- textplate templates (`box`, `text`, `image`) plus a `qr` node, so QR codes can be placed, sized
  and styled anywhere. Multiple QR codes per image are possible.
- Placeholders are filled after parsing, so values like `O'Brien` or `{{x}}` cannot break a template.
- New: rounded boxes, borders, transparent colors, `justify`, shrink-to-fit text, image nodes.

### Reliability
- Invalid templates, colors, fonts, missing columns or images are reported as errors with the
  template path or row number. No panics.
- A failed row no longer aborts the batch; the exit code is 1 if any row failed.
- Output file names are made safe for Linux, macOS and Windows, and duplicates get `_2`, `_3`, ...
- QR codes are drawn with whole-pixel modules and tested by decoding them back.

### Project
- Cargo workspace: `crates/qrgen` (library, no file I/O, builds for `wasm32-unknown-unknown`) and
  `crates/qrgen-cli` (the `qrgen` binary).
- Depends on textplate by git tag, so the repository builds on its own.
- CI: fmt, clippy, tests, wasm builds and the examples.

## v0.2.0

Performance release. Output images look the same as v0.1.0.

### Performance
- Reuse font and glyph caches per worker thread instead of rebuilding them for every row.
- Faster background fill and QR drawing: build QR rows from the module matrix and copy them, instead of blending pixel by pixel.
- Faster PNG encoding: a fixed Paeth filter instead of the adaptive filter. Files are about 4–7% larger.
- Faster text drawing in rust-text-draw: direct buffer writes, and opaque glyph pixels skip blending.
- Faster JPEG base-image decoding (image 0.25 / zune-jpeg).

Benchmarks on 4 threads, compared with v0.1.0:

| Batch | v0.1.0 | v0.2.0 |
|---|---|---|
| 1,000 × QR + caption, 400×430 | 1.36 s | 0.34 s (4.0×) |
| 1,000 × QR card with emoji/text, 1000×1340 | 11.4 s | 1.75 s (6.5×) |
| 200 × multilingual text, 1000×1000 | 1.56 s | 0.48 s (3.2×) |
| 300 × JPEG base image + QR + text, 640×640 | 0.94 s | 0.63 s (1.5×) |

### Changes
- Smaller binaries: `qrgen.wasm` is 4.45 MB → 3.48 MB.
- Base images can be **png, jpeg, webp, gif or bmp**. Other formats (e.g. TIFF) are no longer supported.
- Text shaping now uses HarfRust + skrifa (cosmic-text 0.19). Glyph edges can differ by a pixel or two in anti-aliasing.
- JPEG/WebP base images may decode with tiny rounding differences.
- Removed the debug output that printed every out-of-bounds pixel. The single "Some pixels drawn out of bounds" info line remains.

## v0.1.0

- Add command to use JSON5 configuration files for generation.
