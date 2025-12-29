# fgbdump

fgbdump prints the contents of a [FlatGeobuf](https://flatgeobuf.org/) file in a human-readable terminal user interface.

## Demo

![fgbdump demo](https://github.com/C-Loftus/fgbdump/raw/main/demo.gif)

## Installation

```sh
cargo install --git https://github.com/c-loftus/fgbdump
```

## Limitations

- the map is EPSG:4326 only and I have not implemented projection for the bounding box yet
    - if you would like to see projection support, please open an issue
- the bounding box may have some visual artifacts which cause it to havemore edges than just a pure rectangle
- long metadata values may cause unusual wrap text behavior
