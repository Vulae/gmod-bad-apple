
# [gmod-bad-apple](#)

[![Bad Apple - Garry's Mod Wiremod E2](https://img.youtube.com/vi/nf00VvO-mEk/0.jpg)](https://www.youtube.com/watch?v=nf00VvO-mEk)

# [Setup Instructions](#setup-instructions)

1. Clone repository
2. Run `cargo run -- "media/frames.zip" 30 "media/audio.wav" "media/output.bin" 40 30 10`
3. Create e2 `bad_apple` from `bad_apple.e2`
4. In `bad_apple` e2, import video data
5. Place E2 into world and wire to a digital screen

# [Technical Details](#technical-details)

The video data is encoded into [base64](https://en.wikipedia.org/wiki/Base64) and uploaded as code along side of the decoder e2.

* Video data
    * Format - 1 byte
    * Width - 1 byte
    * Height - 1 byte
    * Frames - 2 bytes
    * FPS - 1 byte
    * Video stream

### [Formats](#formats)

Every video format encodes the difference between frames.  
The reason the difference is used is because setting every pixel every frame is VERY expensive.  
Even though the video data is smaller if we use the frames themselves instead of the difference, the E2 would easily hit tick quota while setting the pixels.  

* [RleSimple](src/video/format/rle_simple.rs) (1)
    * The difference between frames is encoded.
    * Flip flops between off/on with lengths.
        * Eg. `3 1 2 4 1 4 1` will decode to
        ```
        ___X
        __XX
        XX_X
        XXX_
        ```
* [Quadtree](src/video/format/quadtree.rs) (2)
    * The difference between frames is encoded.
    * Splits to sub-trees or leafs, Leafs encode color information.
        * 0 = Split
        * 1 = Leaf (Next bit is if pixel is off/on)
        * Must be leaf if width & height is 1.
        * Each sub-tree/leaf is evaluated from depth first.
            * (Top left, top right, bottom left, bottom right)
        * Eg. `0b010111100110` will decode to
            ```
            __XX
            __XX
            XX_X
            XXX_
            ```
* [RleSimple2](src/video/format/rle_simple2.rs) (3)
    * [RleSimple](src/video/format/rle_simple.rs) but lengths may be encoded into many bytes instead of 1.

Format Table (60x45 @ 7fps)
| Format | Size (Base64) | Total CPU Usage (μs) |
|-|-|-|
| [RleSimple](src/video/format/rle_simple.rs) | 304KiB | 515,026 |
| [Quadtree](src/video/format/quadtree.rs) | 177KiB | 1,170,762 |
| [RleSimple2](src/video/format/rle_simple2.rs) | 291KiB | 436,832 |
