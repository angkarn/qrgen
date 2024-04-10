# QR Code Generator
Tools for generate qrcode image from command line.

## Features
- Add title and font on qrcode image.
- Generate multiple qr from a csv format file.
- Template for custom replace you data before generate.

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
  -t, --title <TITLE>         Put additional title on image
  -f, --format <FORMAT>       Format output (console|png) [default: console]
  -s, --size <SIZE>           Size of image [default: 1024]
  -o, --outdir <OUTDIR>       Output directory [default: output]
  -a, --ass <ADD_SIDE_SPACE>  Side of additional space <top|bottom> [default: bottom]
      --ss <SIZE_SPACE>       Size of additional space (percent of image size) [default: 15]
      --fp <FONT_PATH>        Path of font file
      --fs <FONT_SIZE>        Font size (percentage) [default: 10]
      --tpxy <TITLE_POS_XY>   Positional of additional title (percentage), Empty this will center of additional space
  -h, --help                  Print help
```

### from
`qrgen help from`
```bash
Generate qrcode from a file of list content (csv format)

Usage: qrgen from [OPTIONS] <PATH>

Arguments:
  <PATH>  Path file of list content

Options:
  -i, --icc <INDEX_COLUMN_CONTENT>    Index of column for qr content [default: 0]
      --ict <INDEX_COLUMN_TITLE>      Index of column for additional title
  -t, --template <TEMPLATE>           Template content to replace in list. use `,` for each column eg. `hello {{}}!,,col-3-{{}}` [default: {{}}]
      --sr <SYMBOL_MARK_REPLACE>      Sumbol or substring for mark to replace on templete [default: {{}}]
      --icfn <INDEX_COLUMN_FILENAME>  Index of column to set each file name [default: 0]
  -f, --format <FORMAT>               Format output (console|png) [default: console]
  -s, --size <SIZE>                   Size of image [default: 1024]
  -o, --outdir <OUTDIR>               Output directory [default: output]
  -a, --ass <ADD_SIDE_SPACE>          Side of additional space <top|bottom> [default: bottom]
      --ss <SIZE_SPACE>               Size of additional space (percent of image size) [default: 15]
      --fp <FONT_PATH>                Path of font file
      --fs <FONT_SIZE>                Font size (percentage) [default: 10]
      --tpxy <TITLE_POS_XY>           Positional of additional title (percentage), Empty this will center of additional space
  -h, --help                          Print help
```

## Example
```
1,aaa
2,bbb
3,ccc
```
Generate one and print to console.
```
qrgen gen "Hello World"
```

Save to image file and custom size.
```
qrgen gen "Hello World" -f=png -s=500
```

Generate from list file with set column index of content, filename.
```
qrgen from example_data.csv -f=png --icc=1 --icfn=0
```
![1.png](https://raw.githubusercontent.com/angkarn/qrgen/b29a9bd879691c95664bb18cfbc991fa7e20b6bc/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--icfn%3D0/1.jpg) ![2.png](https://raw.githubusercontent.com/angkarn/qrgen/b29a9bd879691c95664bb18cfbc991fa7e20b6bc/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--icfn%3D0/2.jpg) ![3.png](https://raw.githubusercontent.com/angkarn/qrgen/b29a9bd879691c95664bb18cfbc991fa7e20b6bc/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--icfn%3D0/3.jpg)
```
output/
 1.png
 2.png
 3.png
```

Custom set title and side of additional space. Will auto set title positon to center of additional space.
```
qrgen from example_data.csv -f=png --icc=1 --icfn=0 --ict=1 --ass=top
```
![1.png](https://raw.githubusercontent.com/angkarn/qrgen/b29a9bd879691c95664bb18cfbc991fa7e20b6bc/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--icfn%3D0%20--ict%3D1%20--ass%3Dtop/1.jpg) ![2.png](https://raw.githubusercontent.com/angkarn/qrgen/b29a9bd879691c95664bb18cfbc991fa7e20b6bc/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--icfn%3D0%20--ict%3D1%20--ass%3Dtop/2.jpg) ![3.png](https://raw.githubusercontent.com/angkarn/qrgen/b29a9bd879691c95664bb18cfbc991fa7e20b6bc/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--icfn%3D0%20--ict%3D1%20--ass%3Dtop/3.jpg)

Custom template to replace some text on any data in file. This will custom template column index 0,1 that used for content, filename(default index: 0), title.
```
qrgen from example_data.csv -f=png --icc=1 --ict=1 -t="A{{}},Hello {{}}"
```
![A1.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--ict%3D1%20-t%3D%22A%7B%7B%7D%7D%2CHello%20%7B%7B%7D%7D%22/A1.jpg) ![A2.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--ict%3D1%20-t%3D%22A%7B%7B%7D%7D%2CHello%20%7B%7B%7D%7D%22/A2.jpg) ![A3.png](https://raw.githubusercontent.com/angkarn/qrgen/main/example/assets/from%20example_data.csv%20-f%3Dpng%20--icc%3D1%20--ict%3D1%20-t%3D%22A%7B%7B%7D%7D%2CHello%20%7B%7B%7D%7D%22/A3.jpg)
```
output/
 A1.png
 A2.png
 A3.png
```



