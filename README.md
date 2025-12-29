# fgbdump

fgbdump prints the contents of a [FlatGeobuf](https://flatgeobuf.org/) file in a human-readable terminal user interface.

There are three tabs which can be navigated using the left and right arrow keys:

1. Metadata: shows the metadata of the dataset from the [FlatGeobuf header](https://github.com/flatgeobuf/flatgeobuf/blob/master/src/fbs/header.fbs)
2. Columns: which shows the list of column names in the dataset and associated metadata
3. Map: visualizes the extent of the dataset as a green rectangle over the world map

Within a given tab, you can scroll up and down using the up and down arrow keys or the `j` and `k` keys.

Press `q` or `ctrl-c` to quit the application.

## Demo

![fgbdump demo](https://github.com/C-Loftus/fgbdump/raw/main/demo.gif)

## Installation

_If you would like a pre-built binary for your platform or package manager, please open an issue._

```sh
cargo install --git https://github.com/c-loftus/fgbdump
```

## Limitations

- the map is EPSG:4326 only and I have not implemented projection for the bounding box yet
  - if you would like to see projection support, please open an issue
- the bounding box may have some visual artifacts which cause it to have extra padding around the border and not be a pure rectangle
- long metadata values may cause unusual wrap text behavior
