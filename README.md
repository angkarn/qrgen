# QR Code Generator

Generate QRCode image from command line and library, written in Rust.

## Features

- Add custom text on image.
- Generate multiple qr from a csv format file.
- Custom multiple data e.g. content, text, filename by template!.
- Very small image file.
- Already build target to WASI. [Bonus!](#bonus)

## Download

### Mac

```sh
curl -L -o qrgen https://github.com/angkarn/qrgen/releases/latest/download/qrgen-x86_64-apple-darwin && chmod +x qrgen
```

### Windows (x86_64)

Can be Download from [Release](https://github.com/angkarn/qrgen/releases).

### Other

For other platforms can be clone this repo and build it yourself.

## Uasge

You can use help command to see this.
`qrgen help`

```bash
Usage: qrgen <COMMAND>

Commands:
  gen   Generate qrcode from content
  from  Generate qrcode from a file of list content (csv format)
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### gen

`qrgen help gen`

```bash
Generate qrcode from content

Usage: qrgen gen [OPTIONS] <CONTENT>

Arguments:
  <CONTENT>  Content to generate qrcode

Options:
  -f, --format <FORMAT>               Format output (console|png|base64) [default: console]
  -s, --qr-size <QR_SIZE>             Size of qr [default: 1000]
  -o, --outdir <OUTDIR>               Output directory [default: output]
      --ls <LEFT_SPACE>               Size of left space [default: 0]
      --ts <TOP_SPACE>                Size of top space [default: 0]
      --rs <RIGHT_SPACE>              Size of right space [default: 0]
      --bs <BOTTOM_SPACE>             Size of bottom space [default: 0]
  -r, --ttr <TEMPLATE_TEXT_RENDER>    Template of text render (json5)
      --fp <FONT_PATH>                Paths of font file
      --fs <FONT_SIZE>                Font size (percentage) [default: 50]
      --ecc <ERROR_CORRECTION_LEVEL>  The error correction level in a QR Code symbol. (l|m|q|h) [default: m]
  -h, --help                          Print help
```

### from

`qrgen help from`

```bash
Generate qrcode from a file of list content (csv format)

Usage: qrgen from [OPTIONS] <PATH>

Arguments:
  <PATH>  Path file of list content

Options:
  -c, --tc <TEMPLATE_CONTENT>         Template of qr content [default: {{1}}]
  -n, --tfn <TEMPLATE_FILENAME>       Template filename [default: {{1}}]
  -f, --format <FORMAT>               Format output (console|png|base64) [default: console]
  -s, --qr-size <QR_SIZE>             Size of qr [default: 1000]
  -o, --outdir <OUTDIR>               Output directory [default: output]
      --ls <LEFT_SPACE>               Size of left space [default: 0]
      --ts <TOP_SPACE>                Size of top space [default: 0]
      --rs <RIGHT_SPACE>              Size of right space [default: 0]
      --bs <BOTTOM_SPACE>             Size of bottom space [default: 0]
  -r, --ttr <TEMPLATE_TEXT_RENDER>    Template of text render (json5)
      --fp <FONT_PATH>                Paths of font file
      --fs <FONT_SIZE>                Font size (percentage) [default: 50]
      --ecc <ERROR_CORRECTION_LEVEL>  The error correction level in a QR Code symbol. (l|m|q|h) [default: m]
  -h, --help                          Print help

Template can be use `{{Nunmber of column}}` to replace data of column. And use `{{ROW}}` to replace number of row.
```

## Example

#### gen

Generate one and print to console.

```
qrgen gen "Hello World"
```

Save to image file and custom size.

```
qrgen gen "Hello World" -f png -s 500
```

Add custom text to image both top and bottom. Also handle the new line.
```
qrgen gen "Hello World" -f png -t "QR Generator" -b "Hello\nWorld"
```
![qr](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/text_top_bottom/qr.jpg)

#### from

```
1,A
2,B
3,C
```

Generate from list file with custom template of content, filename.
```
qrgen from example_data.csv -f png --tc "{{0}}:{{1}}" --tfn "no_{{0}}"
```
![no_1](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/template_content_filename/no_1.jpg) ![no_2](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/template_content_filename/no_2.jpg) ![no_3](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/template_content_filename/no_3.jpg)
```
output/
 no_1.png
 no_2.png
 no_3.png
```

Add custom text of both side.
```
qrgen from example_data.csv -f png --ttt "QR Gen" --ttb "#{{0}}: {{1}}"
```
![1](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/list_custom_text/1.jpg) ![2](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/list_custom_text/2.jpg) ![3](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/list_custom_text/3.jpg)

Custom font.
```
qrgen gen "QR Generate" -f png -b "QR Generate" --fp "fonts/Bangers-Regular.ttf"
```
![qr](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/custom_font/qr.jpg)

```
qrgen gen "1234" -f png -b "*1234*" --fp "fonts/LibreBarcode39-Regular.ttf" --fs 20
```
![qr](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/custom_font_barcode/qr.jpg)

## Bonus!
Web demo uses wasm file from build target to WebAssembly (WASI) `wasm32-wasi`. It can run on client browser.
[Demo](https://qrgen-rs.pages.dev)

## Build
You can build by follow basic rust tool like cargo.


