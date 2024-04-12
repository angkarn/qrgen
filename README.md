# QR Code Generator

Tools for generate qrcode image from command line.

## Features

- Add text on image.
- Generate multiple qr from a csv format file.
- Custom multiple data by template!.

## Download

### Mac

```sh
curl -L -o qrgen https://github.com/angkarn/qrgen/releases/download/v0.1.0-dev/qrgen-x86_64-apple-darwin && chmod +x qrgen
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
```

### gen

`qrgen help gen`

```bash
Generate qrcode from content

Usage: qrgen gen [OPTIONS] <CONTENT>

Arguments:
  <CONTENT>  Content to generate qrcode

Options:
  -t, --top-text <TOP_TEXT>         Text on top of image [default: ]
  -b, --bottom-text <BOTTOM_TEXT>   Text on bottom of image [default: ]
  -f, --format <FORMAT>             Format output (console|png) [default: console]
  -s, --size <SIZE>                 Size of image [default: 1024]
  -o, --outdir <OUTDIR>             Output directory [default: output]
      --ts <TOP_SPACE>              Size of top space (percent of qr size) [default: 15]
      --bs <BOTTOM_SPACE>           Size of bottom space (percent of qr size) [default: 15]
      --fp <FONT_PATH>              Path of font file
      --fs <FONT_SIZE>              Font size (percentage) [default: 10]
      --atls <ADD_TEXT_LINE_SPACE>  Add text line space (percentage) [default: 0]
      --nrts                        Flag to ignore auto reduce text size
  -h, --help                        Print help
```

### from

`qrgen help from`

```bash
Generate qrcode from a file of list content (csv format)

Usage: qrgen from [OPTIONS] <PATH>

Arguments:
  <PATH>  Path file of list content

Options:
  -t, --tc <TEMPLATE_CONTENT>       Template content [default: {{0}}]
      --ttt <TEMPLATE_TEXT_TOP>     Template for text on top [default: ]
      --ttb <TEMPLATE_TEXT_BOTTOM>  Template for text on bottom [default: ]
      --tfn <TEMPLATE_FILENAME>     Template filename [default: {{0}}]
  -f, --format <FORMAT>             Format output (console|png) [default: console]
  -s, --size <SIZE>                 Size of image [default: 1024]
  -o, --outdir <OUTDIR>             Output directory [default: output]
      --ts <TOP_SPACE>              Size of top space (percent of qr size) [default: 15]
      --bs <BOTTOM_SPACE>           Size of bottom space (percent of qr size) [default: 15]
      --fp <FONT_PATH>              Path of font file
      --fs <FONT_SIZE>              Font size (percentage) [default: 10]
      --atls <ADD_TEXT_LINE_SPACE>  Add text line space (percentage) [default: 0]
      --nrts                        Flag to ignore auto reduce text size
  -h, --help                        Print help

TEMPLATE: Can be use {{INDEX_COLUMN}} to replace from data (Starting at 0). eg. `Hello {{1}}` is replace {{1}} to data of index 1 on row.
```

## Example

#### gen

Generate one and print to console.

```
qrgen gen "Hello World"
```

Save to image file and custom size.

```
qrgen gen "Hello World" -f=png -s=500
```

Add custom text to image both top and bottom. Also handle the new line.
```
qrgen gen "Hello World" -f=png -t="QR Generator" -b="Hello\nWorld"
```
![qr.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/gen%20%22Hello%20World%22%20-f%3Dpng%20-t%3D%22QR%20Generator%22%20-b%3D%22Hello%5CnWorld%22/qr.jpg)

#### from

```
1,A
2,B
3,C
```

Generate from list file with custom template of content, filename.
```
qrgen from example_data.csv -f=png --tc="{{0}}:{{1}}" --tfn="no_{{0}}"
```
![no_1.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--tc%3D%22%7B%7B0%7D%7D%3A%7B%7B1%7D%7D%22%20--tfn%3D%22no_%7B%7B0%7D%7D%22/no_1.jpg) ![no_2.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--tc%3D%22%7B%7B0%7D%7D%3A%7B%7B1%7D%7D%22%20--tfn%3D%22no_%7B%7B0%7D%7D%22/no_2.jpg) ![no_3.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--tc%3D%22%7B%7B0%7D%7D%3A%7B%7B1%7D%7D%22%20--tfn%3D%22no_%7B%7B0%7D%7D%22/no_3.jpg)
```
output/
 no_1.png
 no_2.png
 no_3.png
```

Add custom text of both side.
```
qrgen from example_data.csv -f=png --ttt="QR Gen" --ttb="#{{0}}: {{1}}"
```
![1.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--ttt%3D%22QR%20Gen%22%20--ttb%3D%22%23%7B%7B0%7D%7D%3A%20%7B%7B1%7D%7D%22/1.jpg) ![2.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--ttt%3D%22QR%20Gen%22%20--ttb%3D%22%23%7B%7B0%7D%7D%3A%20%7B%7B1%7D%7D%22/2.jpg) ![3.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--ttt%3D%22QR%20Gen%22%20--ttb%3D%22%23%7B%7B0%7D%7D%3A%20%7B%7B1%7D%7D%22/3.jpg)



