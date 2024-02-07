
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
    * Frames - 2 bytes
    * FPS - 1 byte
    * Video stream

### [Formats](#formats)

* [RleSimple](src/format/rle_simple.rs) (1)
    * The difference between frames is encoded.
    * Flip flops between off/on with lengths.
        * Eg. `3 1 2 4 1 4 1` will decode to
        ```
        ___X
        __XX
        XX_X
        XXX_
        ```
* [Quadtree](src/format/quadtree.rs) (2)
    * The difference between frames is encoded.
    * Splits to sub-trees or leafs, Leafs encode color information.
        * 0 = Split
        * 1 = Leaf (Next bit is if pixel is off/on)
        * Each sub-tree/leaf is evaluated from depth first.
            * (Top left, top right, bottom left, bottom right)
        * Eg. `0b0101111010111110` will decode to
            ```
            __XX
            __XX
            XX_X
            XXX_
            ```
        * Due to some current limitations while splitting quadtree, there may be some artifacts while splitting small sections.
            * Eg. 1x2 will not split, as 2 of the nodes will be 0 sized.
            * This may be fixed later on, by adding a 4 or 2 number split nodes.