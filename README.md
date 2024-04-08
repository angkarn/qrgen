# QR Code Generator
Tools for generate qrcode image from command line.

## Features
- Add additional space with custom title and set position on your qrcode image.
- Generate multiple images from a csv format file. 
- Customize template for replace any you data (eg. QR content, Title) from list file.

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
      --fp <FONT_PATH>                Path of font file (default use "IBMPlexSansThaiLooped-Light")
      --fs <FONT_SIZE>                Font size (percentage) [default: 10]
      --tpxy <TITLE_POS_XY>           Positional of additional title (percentage), Empty this will center of additional space
  -h, --help                          Print help
```

## Example

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

Custom set title and side of additional space. Will auto set title positon to center of additional space.
```
qrgen from example_data.csv -f=png --icc=1 --icfn=0 --ict=1 --ass=top
```

Custom template to replace some text on any data in file. This example will custom column index 0,1 that used for content, filename, title
```
qrgen from example_data.csv -f=png --icc=1 --icfn=0 --ict=1 --ass=top -t="A{{}},Hello {{}}"
```



