# QR Code Generator

Generate QRCode image and draw customizable texts (font, template on json) via command line and library, written in Rust.

## Features

- Draw custom text, font, .etc on image via json template!.
- Generate multiple QR from a csv format file.
- Set template from data list e.g. qr content, draw, filename, set base image.

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
  gen   Generate one qrcode
  from  Generate multiple qrcode from a file of list content (csv format)
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
Generate one qrcode

Usage: qrgen gen [OPTIONS] <CONTENT>

Arguments:
  <CONTENT>  Content to generate qrcode

Options:
  -f, --format <FORMAT>               Format output (console|png|base64) [default: console]
  -b, --base_image <BASE_IMAGE>       Path image file for set base image. It will ignore config image width, height (also work with data template)
  -1, --qr_color_1 <QR_COLOR_1>       Color of QR (1 like black) [default: 000000ff]
  -0, --qr_color_0 <QR_COLOR_0>       Color of QR (0 like white) [default: ffffffff]
      --fill <FILL_COLOR>             fill background color [default: ffffffff]
  -w, --image_width <IMAGE_WIDTH>     Size of image width (pixel) [default: 1000]
  -h, --image_height <IMAGE_HEIGHT>   Size of image height (pixel) (default value is image width)
  -s, --qr_size <QR_SIZE>             Size of qr (pixel) (default value is image width)
  -x, --pos_x <POS_QR_X>              Start position qr x axis (pixel) [default: 0]
  -y, --pos_y <POS_QR_Y>              Start position qr y axis (pixel) [default: 0]
  -d, --td <TEMPLATE_DRAW>            Template of draw (json5)
      --fp <FONT_PATH>                Paths of font files
      --fs <FONT_SIZE>                Default Font size (percentage of image width) [default: 3]
  -o, --outdir <OUTDIR>               Output directory [default: output]
      --ecc <ERROR_CORRECTION_LEVEL>  The error correction level in a QR Code symbol. (l|m|q|h) [default: m]
```

### from

`qrgen help from`

```bash
Generate multiple qrcode from a file of list content (csv format)

Usage: qrgen from [OPTIONS] <PATH>

Arguments:
  <PATH>  Path file of list content

Options:
  -c, --tc <TEMPLATE_CONTENT>         Template of qr content [default: {{1}}]
  -n, --tfn <TEMPLATE_FILENAME>       Template filename [default: {{1}}]
  -f, --format <FORMAT>               Format output (console|png|base64) [default: console]
  -b, --base_image <BASE_IMAGE>       Path image file for set base image. It will ignore config image width, height (also work with data template)
  -1, --qr_color_1 <QR_COLOR_1>       Color of QR (1 like black) [default: 000000ff]
  -0, --qr_color_0 <QR_COLOR_0>       Color of QR (0 like white) [default: ffffffff]
      --fill <FILL_COLOR>             fill background color [default: ffffffff]
  -w, --image_width <IMAGE_WIDTH>     Size of image width (pixel) [default: 1000]
  -h, --image_height <IMAGE_HEIGHT>   Size of image height (pixel) (default value is image width)
  -s, --qr_size <QR_SIZE>             Size of qr (pixel) (default value is image width)
  -x, --pos_x <POS_QR_X>              Start position qr x axis (pixel) [default: 0]
  -y, --pos_y <POS_QR_Y>              Start position qr y axis (pixel) [default: 0]
  -d, --td <TEMPLATE_DRAW>            Template of draw (json5)
      --fp <FONT_PATH>                Paths of font files
      --fs <FONT_SIZE>                Default Font size (percentage of image width) [default: 3]
  -o, --outdir <OUTDIR>               Output directory [default: output]
      --ecc <ERROR_CORRECTION_LEVEL>  The error correction level in a QR Code symbol. (l|m|q|h) [default: m]

Template can be use `{{Number of column}}` to replace data of column. And use `{{ROW}}` to replace number of row.
```

## Example

#### gen

Generate one qr and print to console.

```bash
qrgen gen "Hello World"
```

Save to image file and custom size.

```bash
qrgen gen "Hello World" -f=png -w=400
```

Simple Draw text to image.

```bash
qrgen gen abc123 -f=png --fs=10 -w=400 -h=450 --td="[{a:1,p:2,ts:[{t:'abc123'}]}]"
```

## ![qr](https://raw.githubusercontent.com/angkarn/qrgen/main/example/simple_draw/qr.png)

#### from

Generate from csv list and set template of QR content, Draw, Filename. (file on repo)

```bash
qrgen from example_data.csv -f=png -w=1000 -h=1340 -y=150 --tfn="no_{{ROW}}" --tc="{{ROW}}|{{1}}" -d="[{a:1,ts:[{f:10,fs:8,t:'🌈'},{f:2,fs:11,c:'C63658',t:'Hello '},{f:1,fs:12,t:'*world*'}]},{y:85,h:15,fill:'435058',c:'ffffff',fs:5,wi:[{p:1,ml:5,fs:5,ts:[{t:'Infomation\nNo. {{ROW}}'}]},{y:16,a:2,mr:12,c:'ffffff',ts:[{t:'{{1}}'}]},{y:16,a:2,mr:4,c:'ffffff',ts:[{f:3,t:'👤'}]},{y:50,a:2,mr:12,c:'ffffff',ts:[{t:'{{2}}'}]},{y:50,mr:4,a:2,c:'ffffff',ts:[{f:3,t:'✉️'}]}]}]" --fp=fonts/LibreBarcode39Text-Regular.ttf,fonts/Monofett-Regular.ttf,fonts/noto-emoji-v53-emoji-regular.ttf,fonts/noto-sans-arabic-v28-arabic-regular.ttf,fonts/noto-sans-devanagari-v26-devanagari-regular.ttf,fonts/noto-sans-jp-v53-japanese-regular.ttf,fonts/noto-sans-kr-v36-korean-regular.ttf,fonts/noto-sans-sc-v37-chinese-simplified-regular.ttf,fonts/noto-sans-thai-v25-thai-regular.ttf,fonts/NotoColorEmoji.ttf
```

<img src="https://raw.githubusercontent.com/angkarn/qrgen/main/example/template_qr_filename_draw/no_1.png" alt="no_1.png" width="300px"/>

```
output/
no_1.png
no_2.png
no_3.png
```

---

### Fun!

No QR

```bash
qrgen from example_data_markup.csv -f=png -w=1000 -h=1000 -s=0 --fs=2.4 --tfn="hello_world" --fp=fonts/noto-sans-thai-v25-thai-regular.ttf,fonts/noto-sans-arabic-v28-arabic-regular.ttf,fonts/noto-sans-jp-v53-japanese-regular.ttf,fonts/noto-sans-kr-v36-korean-regular.ttf,fonts/noto-sans-sc-v37-chinese-simplified-regular.ttf,fonts/noto-sans-devanagari-v26-devanagari-regular.ttf,fonts/NotoColorEmoji.ttf,fonts/Monofett-Regular.ttf,fonts/LibreBarcode39Text-Regular.ttf -d="[{fill:'e9e0d4'},{y:3,a:1,c:'000000',ts:[{fs:6,t:'{{1}}'},{f:8,fs:8,c:'C63658',t:'{{2}} '},{f:9,fs:8,t:'*{{3}}*'}]},{a:1,y:18,fs:4.5,ts:[{t:'{{4}}'}]},{y:28,h:0.3,ml:30,mr:30,fill:'b1b2b3'},{y:30,c:'ff999a',ts:[{t:'{{5}}'},{c:'0c87a5',t:' {{6}}'}]},{y:40,a:2,ts:[{fs:2,t:'{{5}} {{6}}'}]},{y:52,ml:3,mr:3,p:1,w:50,h:40,ts:[{c:'000000',t:'{{5}} {{6}}'}]},{x:50,y:52,w:50,h:45,ml:3,mr:3,fill:'8d8282',wi:[{ml:3,mr:3,mb:3,mt:3,fill:'36454F',wi:[{p:2,ml:1,mr:1,ts:[{c:'ffffff',t:'{{5}} {{6}}'}]}]}]},{ts:[{c:'e74c3c',t:'a:0'},{c:'2ecc71',t:' p:0'}]},{a:1,ts:[{c:'5dade2',t:'a:1'},{c:'2ecc71',t:' p:0'}]},{a:2,ts:[{c:'f39c12',t:'a:2'},{c:'2ecc71',t:' p:0'}]},{p:1,ts:[{c:'e74c3c',t:'a:0'},{c:'71569b',t:' p:1'}]},{a:1,p:1,ts:[{c:'5dade2',t:'a:1'},{c:'71569b',t:' p:1'}]},{a:2,p:1,ts:[{c:'f39c12',t:'a:2'},{c:'71569b',t:' p:1'}]},{p:2,ts:[{c:'e74c3c',t:'a:0'},{c:'f17ba3',t:' p:2'}]},{a:1,p:2,ts:[{c:'5dade2',t:'a:1'},{c:'f17ba3',t:' p:2'}]},{a:2,p:2,ts:[{c:'f39c12',t:'a:2'},{c:'f17ba3',t:' p:2'}]}]"
```

## <img src="https://raw.githubusercontent.com/angkarn/qrgen/main/example/hello_world/hello_world.png" alt="no_1" width="500px"/>

Set base image, custom qr color.

```bash
qrgen gen "travel is my therapy\!" -b="base_image.jpg" -f=png -s=140 -x=435 -y=20 -0="ffffff4d" -1="00000080" --fp=fonts/knewave-v14-latin-regular.ttf -d="[{x:9,y:35,f:1,fs:7,c:'ffffff',ts:[{t:'Travel'}]},{x:15,y:50,fs:3,c:'ffffff',ts:[{t:'is my'}]},{x:4,y:53.1,f:1,fs:7,c:'ffffff',ts:[{t:'Therapy!'}]}]"
```

<img src="https://raw.githubusercontent.com/angkarn/qrgen/main/example/set_base_image/qr.png" alt="set_base_image" width="600px"/>

---

Set base image by data template.

```bash
qrgen from "example/set_base_image/from/data.csv" -b="example/set_base_image/from/{{2}}" -f=png -s=0 --fp=fonts/paytone-one-v23-latin-regular.ttf -d="[{a:1,p:1,f:1,fs:16,c:'ffffff',ts:[{t:'{{1}}'}]}]"
```

<img src="https://raw.githubusercontent.com/angkarn/qrgen/main/example/set_base_image/from/output/Bridge.png" alt="Bridge" width="300px"/>
<img src="https://raw.githubusercontent.com/angkarn/qrgen/main/example/set_base_image/from/output/Koi%20carp.png" alt="Koi carp" width="300px"/>
<img src="https://raw.githubusercontent.com/angkarn/qrgen/main/example/set_base_image/from/output/Moon.png" alt="Moon" width="300px"/>

---

#### WASI

[Demo](https://qrgen-browser-demo.pages.dev) uses wasm file from build target to WebAssembly `wasm32-wasi`, By running on client browser. [project link](https://github.com/angkarn/qrgen-browser-demo)

## Build

This project uses dependency from [rust-text-draw](https://github.com/angkarn/rust-text-draw). Please clone this repo before.
