// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::{QoiDecoder, QoiError, OUTPUT_OFFSET};
const OP_RGB: u8 = 0xfe;
const OP_RGBA: u8 = 0xff;
const OP_MASK: u8 = 0xc0;
const OP_DIFF: u8 = 0x40;
const OP_LUMA: u8 = 0x80;
const OP_RUN: u8 = 0xc0;
fn hash(pixel: [u8; 4]) -> u32 {
    (u32::from(pixel[0]) * 3
        + u32::from(pixel[1]) * 5
        + u32::from(pixel[2]) * 7
        + u32::from(pixel[3]) * 11)
        & 63
}
impl QoiDecoder {
    pub(super) fn decode_memory(
        &mut self,
        offset: usize,
        length: usize,
        pixels: i32,
    ) -> Result<(), QoiError> {
        self.memory
            .get_mut(..256)
            .ok_or(QoiError::MemoryAccess)?
            .fill(0);
        // Uint8Array input has a wasm32 address/length in the original decoder.
        let offset = u32::try_from(offset).map_err(|_| QoiError::InputOffset)?;
        let length = u32::try_from(length).map_err(|_| QoiError::InputOffset)?;
        if length < 23 {
            return Ok(());
        }
        let mut read = offset.wrapping_add(14);
        let end = offset.wrapping_add(length).wrapping_sub(8);
        let mut write = OUTPUT_OFFSET as u32;
        let write_end = write.wrapping_add((pixels as u32).wrapping_mul(4));
        let mut pixel = [0, 0, 0, 255];
        while read < end {
            let opcode = self.byte(read)?;
            read = read.wrapping_add(1);
            let kind = opcode & OP_MASK;
            let low = opcode & 63;
            if kind == 0 {
                pixel = self.pixel(u32::from(low) * 4)?;
                self.set_pixel(write, pixel)?;
                write = write.wrapping_add(4);
            } else if kind == OP_RUN && opcode < OP_RGB {
                self.set_pixel(hash(pixel) * 4, pixel)?;
                let mut repeat = low;
                loop {
                    self.set_pixel(write, pixel)?;
                    write = write.wrapping_add(4);
                    let more = repeat != 0;
                    repeat = repeat.wrapping_sub(1);
                    if !more || write >= write_end {
                        break;
                    }
                }
            } else {
                if opcode == OP_RGB {
                    for channel in pixel.iter_mut().take(3) {
                        *channel = self.byte(read)?;
                        read = read.wrapping_add(1);
                    }
                } else if opcode == OP_RGBA {
                    pixel = self.pixel(read)?;
                    read = read.wrapping_add(4);
                } else if kind == OP_DIFF {
                    pixel[0] = pixel[0].wrapping_add((low >> 4) & 3).wrapping_sub(2);
                    pixel[1] = pixel[1].wrapping_add((low >> 2) & 3).wrapping_sub(2);
                    pixel[2] = pixel[2].wrapping_add(low & 3).wrapping_sub(2);
                } else if kind == OP_LUMA {
                    let second = self.byte(read)?;
                    read = read.wrapping_add(1);
                    let green = low.wrapping_sub(32);
                    pixel[0] = pixel[0]
                        .wrapping_add(green)
                        .wrapping_sub(8)
                        .wrapping_add(second >> 4);
                    pixel[1] = pixel[1].wrapping_add(green);
                    pixel[2] = pixel[2]
                        .wrapping_add(green)
                        .wrapping_sub(8)
                        .wrapping_add(second & 15);
                }
                self.set_pixel(hash(pixel) * 4, pixel)?;
                self.set_pixel(write, pixel)?;
                write = write.wrapping_add(4);
            }
        }
        Ok(())
    }
}
