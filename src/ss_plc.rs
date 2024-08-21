use std::{
    fs::{self, File},
    io::Write,
};

use glam::Vec4;

use crate::collision::plc::{PLCEntry, PLC};

/** Bit Field Definitions
 * Code 0:
 *  - 0x0000_003F |             |
 *  - 0x0000_0FC0 |             |
 *  - 0x0000_4000 |             | 8034b7b0
 *  - 0x0000_8000 |             | 8034b7b0
 *  - 0x0002_0000 |             | 8034b7b0
 *  - 0x0008_0000 |             | 8034b7b0
 *  - 0x0010_0000 |             | 8034b7b0
 *  - 0x0020_0000 |             |
 *  - 0x0040_0000 |             |
 *  - 0x0080_0000 |             | 8034b7b0
 *  - 0x0100_0000 |             | 8034b7b0
 *  - 0x1000_0000 |             |
 *  - 0x4000_0000 |             | 8034b7b0
 *
 * Code 1:
 *  - 0x0000_00FF |             |
 *  - 0x0000_0F00 |             |
 *  - 0x000E_0000 |             |
 *  - 0x01F0_0000 | Ground Type |
 *  - 0x0200_0000 |             |
 *  - 0x0400_0000 |             | 8034b7b0
 *  - 0x0800_0000 |             | 8034b7b0
 *  - 0xF000_0000 |             | 8034b7b0
 *
 * Code 2:
 *  - 0x0000_00FF |             |
 *  - 0x0000_FF00 |             |
 *  - 0x00FF_0000 |             |
 *  - 0xFF00_0000 |             |
 *
 * Code 3:
 *  - 0x0000_07E0 | limited to 0-31 |
 *
 * Code 4:
 *  - 0xFFFF_FFFF | | 8034b4c0
 */

impl PLCEntry {
    pub fn get_color(&self) -> Vec4 {
        let val = (self.codes[1] >> 0x14) & 0x1F;
        return Vec4::splat(val as f32 / 31f32).with_w(1.0f32);
        match val {
            0..32 => Vec4::splat(self.codes[1] as f32 / 31f32).with_w(1.0f32),
            _ => Vec4::new(0.0f32, 0.0f32, 0.0f32, 0.0f32),
        }
    }
}

impl PLC {
    pub fn dump(&self, file: &mut fs::File) {
        for entry in &self.entries {
            let [code0, code1, code2, code3, code4] = entry.codes;
            let _ = file.write(
                format!("{code0:08X?} {code1:08X?} {code2:08X?} {code3:08X?} {code4:08X?}\n")
                    .as_bytes(),
            );
        }
    }
}
