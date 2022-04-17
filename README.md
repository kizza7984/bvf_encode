# bvf_encode

Takes a directory of input images and outputs a video in a 1-bit colour format ("BVF") designed to be simple and easy to decode, for displaying on constrained hardware such as a [graphing calculator](https://github.com/kizza7984/bvf_play).

## BVF

Each frame is broken up into horizontal lines, the content of which are expressed as vectors from a start pixel to an end pixel preceded by a byte indicating the number of vectors,
for example a blank line would be a `0` byte, and a black line would be: `1`, `0`, `127` where the image width is 128, meaning that there is one vector from the start of the line to the end of the line.
There are no separators indicating frames, when decoding this is inferred from the vertical resolution.

Metadata bytes (start of the file):
```
0 Frame rate
1 | Frame count (Little Endian)
2 |
3 |
4 |
5 Horizontal resolution
6 Vertical resolution
7+ <Lines>
...
```
Line bytes:
```
0 Number of vectors in the line
1 | <Vector>
2 |
...
```
Vector bytes:
```
0 Start pixel
1 End pixel
```

## Usage

Images in the input directory must follow the format `frame{n}.png`. Outputs to `output.bvf` by default unless set with `--output/-o`.
```
bvf_encode <INPUT_DIRECTORY> [OPTIONS] --frame-count <FRAME_COUNT> --frame-rate <FRAME_RATE> --horizontal-resolution <HORIZONTAL_RESOLUTION> --vertical-resolution <VERTICAL_RESOLUTION>
```
Examples:
```
bvf_encode frames --frame-count 2191 --frame-rate 15 --horizontal-resolution 128 --vertical-resolution 64
bvf_encode frames -c 2191 -r 15 -h 128 -v 64
bvf_encode frames -c 2191 -r 15 -h 128 -v 64 --output my_output_file.bvf
```
