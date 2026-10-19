# Changelog

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
