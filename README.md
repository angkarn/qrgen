# QR Code Generator

Generate QRCode image and draw customizable texts (font, template on json) via command line and library, written in Rust.

## Features

- Draw custom text and font on image via json template!.
- Generate multiple QR from a csv format file.
- Set template from data list e.g. qr content, draw, filename.
- Optimal size and performance of image files.
- [Bonus!](#bonus) Already and Demo build target to WASI.

## Download

### Mac (Apple Silicon)

```sh
curl -L -o qrgen https://github.com/angkarn/qrgen/releases/latest/download/qrgen-aarch64-apple-darwin && chmod +x qrgen
```

### Mac (Intel)

```sh
curl -L -o qrgen https://github.com/angkarn/qrgen/releases/latest/download/qrgen-x86_64-apple-darwin && chmod +x qrgen
```

### Windows (x86_64)

Can be Download from [Release](https://github.com/angkarn/qrgen/releases).

### Other

For other platforms can be clone this repo and try build it.

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
      --help
          Print help

  -V, --version
          Print version
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
  -s, --qr_size <QR_SIZE>             Size of qr [default: 1000]
  -w, --image_width <IMAGE_WIDTH>     Size of image width (default value from qr size)
  -h, --image_height <IMAGE_HEIGHT>   Size of image height (default value from qr size)
  -o, --outdir <OUTDIR>               Output directory [default: output]
  -x, --pos_x <POS_QR_X>              Start position qr x axis [default: 0]
  -y, --pos_y <POS_QR_Y>              Start position qr y axis [default: 0]
  -d, --td <TEMPLATE_DRAW>            Template of text render (json5)
      --fp <FONT_PATH>                Paths of font file
      --fs <FONT_SIZE>                Font size (percentage) [default: 50]
      --ecc <ERROR_CORRECTION_LEVEL>  The error correction level in a QR Code symbol. (l|m|q|h) [default: m]
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
  -s, --qr_size <QR_SIZE>             Size of qr [default: 1000]
  -w, --image_width <IMAGE_WIDTH>     Size of image width (default value from qr size)
  -h, --image_height <IMAGE_HEIGHT>   Size of image height (default value from qr size)
  -o, --outdir <OUTDIR>               Output directory [default: output]
  -x, --pos_x <POS_QR_X>              Start position qr x axis [default: 0]
  -y, --pos_y <POS_QR_Y>              Start position qr y axis [default: 0]
  -d, --td <TEMPLATE_DRAW>            Template of text render (json5)
      --fp <FONT_PATH>                Paths of font file
      --fs <FONT_SIZE>                Font size (percentage) [default: 50]
      --ecc <ERROR_CORRECTION_LEVEL>  The error correction level in a QR Code symbol. (l|m|q|h) [default: m]

Template can be use `{{Number of column}}` to replace data of column. And use `{{ROW}}` to replace number of row.
```

## Example

#### gen

Generate one qr and print to console.

```
qrgen gen "Hello World"
```

Save to image file and custom size.

```
qrgen gen "Hello World" -f=png -s=400
```

Simple Draw text to image.

```
qrgen gen abc123 -f=png --fs=40 -s=400 -h=450 --td="[{a:1,p:2,ts:[{t:'abc123'}]}]"
```

![qr](https://raw.githubusercontent.com/angkarn/qrgen/main/example/simple_draw/qr.png)

#### from

Generate from csv list and set template of QR content, Draw, Filename. (file on repo)

```
qrgen from example_data.csv -f=png -s=1000 -y=150 -w=1000 -h=1340 --tfn="no_{{ROW}}" --tc="{{ROW}}|{{1}}" -d="[{y:20,h:140,a:1,p:1,ts:[{f:10,fs:70,t:'🌈'},{f:2,fs:80,c:'C63658',t:'Hello '},{f:1,fs:100,t:'*world*'}]},{y:1160,h:180,fill:'435058',c:'ffffff',f:40,fs:40,wi:[{p:1,ml:50,fs:45,ts:[{t:'Infomation\nNo. {{ROW}}'}]},{y:30,a:2,mr:120,c:'ffffff',ts:[{t:'{{1}}'}]},{y:30,a:2,mr:40,c:'ffffff',ts:[{f:3,t:'👤'}]},{y:90,a:2,mr:120,c:'ffffff',ts:[{t:'{{2}}'}]},{y:90,a:2,x:-40,c:'ffffff',ts:[{f:3,t:'✉️'}]}]}]" --fp=fonts/LibreBarcode39Text-Regular.ttf,fonts/Monofett-Regular.ttf,fonts/noto-emoji-v53-emoji-regular.ttf,fonts/noto-sans-arabic-v28-arabic-regular.ttf,fonts/noto-sans-devanagari-v26-devanagari-regular.ttf,fonts/noto-sans-jp-v53-japanese-regular.ttf,fonts/noto-sans-kr-v36-korean-regular.ttf,fonts/noto-sans-sc-v37-chinese-simplified-regular.ttf,fonts/noto-sans-thai-v25-thai-regular.ttf,fonts/NotoColorEmoji.ttf
```

![no_1](https://raw.githubusercontent.com/angkarn/qrgen/main/example/template_qr_filename_draw/no_1.png)

```
output/
 no_1.png
 no_2.png
 no_3.png
```

## Bonus!

[Demo](https://qrgen-browser-demo.pages.dev) and [repo](https://github.com/angkarn/qrgen-browser-demo) uses wasm file from build target to WebAssembly `wasm32-wasi`, By running on client browser.

## Build

This project uses dependency from [rust-text-draw](https://github.com/angkarn/rust-text-draw). Please clone this repo before.
