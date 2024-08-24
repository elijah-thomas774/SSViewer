use std::{fs, io::Write};

use glam::Vec4;

use crate::file_formats::{PLCEntry, PLC};

#[derive(Debug, Clone)]
pub struct ShiftMask {
    pub code_idx: usize,
    pub shift: u32,
    pub mask: u32,
}

impl ShiftMask {
    const fn new(code_idx: usize, shift: u32, mask: u32) -> Self {
        Self {
            code_idx,
            shift,
            mask,
        }
    }
}
pub enum EntryType {
    Norm,
    Range(ShiftMask),
    Single(ShiftMask),
}

use EntryType::*;
pub const ENTRY_FILTER: [EntryType; 32] = [
    EntryType::Norm,
    Range(ShiftMask::new(0, 00, 0x3F)), //  - 0x0000_003F |                 |
    Range(ShiftMask::new(0, 06, 0xFF)), //  - 0x0000_3FC0 |                 |
    Single(ShiftMask::new(0, 14, 0x1)), //  - 0x0000_4000 |  Pass Obj       |
    Single(ShiftMask::new(0, 15, 0x1)), //  - 0x0000_8000 |  Pass Camera    |
    Single(ShiftMask::new(0, 16, 0x1)), //  - 0x0001_0000 |  Pass Link      |
    Single(ShiftMask::new(0, 17, 0x1)), //  - 0x0002_0000 |  Pass Arrow     |
    Single(ShiftMask::new(0, 18, 0x1)), //  - 0x0004_0000 |  Pass Slingshot |
    Single(ShiftMask::new(0, 19, 0x1)), //  - 0x0008_0000 |  Pass Beetle    |
    Single(ShiftMask::new(0, 20, 0x1)), //  - 0x0010_0000 |  Pass Clawshot  |
    Single(ShiftMask::new(0, 21, 0x1)), //  - 0x0020_0000 |  Pass Z-Target  |
    Single(ShiftMask::new(0, 22, 0x1)), //  - 0x0040_0000 |  Pass Shadow    |
    Single(ShiftMask::new(0, 23, 0x1)), //  - 0x0080_0000 |  Pass Bomb      |
    Single(ShiftMask::new(0, 24, 0x1)), //  - 0x0100_0000 |  Pass Whip      |
    Range(ShiftMask::new(0, 28, 0x3)),  //  - 0x3000_0000 |                 |
    Single(ShiftMask::new(0, 30, 0x1)), //  - 0x4000_0000 |                 | 8034b7b0
    Single(ShiftMask::new(0, 31, 0x1)), //  - 0x8000_0000 |                 |
    Range(ShiftMask::new(1, 0, 0xFF)),  //  - 0x0000_00FF |                 |
    Range(ShiftMask::new(1, 8, 0xF)),   //  - 0x0000_0F00 |                 |
    Range(ShiftMask::new(1, 17, 0x7)),  //  - 0x000E_0000 |                 |
    Range(ShiftMask::new(1, 20, 0x1F)), //  - 0x01F0_0000 | Ground Type     |
    Single(ShiftMask::new(1, 25, 0x1)), //  - 0x0200_0000 |                 |
    Single(ShiftMask::new(1, 26, 0x1)), //  - 0x0400_0000 |                 | 8034b7b0
    Single(ShiftMask::new(1, 27, 0x1)), //  - 0x0800_0000 |                 | 8034b7b0
    Range(ShiftMask::new(1, 28, 0xF)),  //  - 0xF000_0000 |                 | 8034b7b0
    Range(ShiftMask::new(2, 0, 0xFF)),  //  - 0x0000_00FF |                 |
    Range(ShiftMask::new(2, 8, 0xFF)),  //  - 0x0000_FF00 |                 |
    Range(ShiftMask::new(2, 16, 0xFF)), //  - 0x00FF_0000 |                 |
    Range(ShiftMask::new(2, 24, 0xFF)), //  - 0xFF00_0000 |                 |
    Range(ShiftMask::new(3, 0, 0x1F)),  //  - 0x0000_001F | limited to 0-31 | 0xC is Vines
    Range(ShiftMask::new(3, 5, 0x3F)),  //  - 0x0000_07E0 | limited to 0-31 | 0xC is Vines
    Range(ShiftMask::new(4, 0, 0xFFFFFFFF)), //  - 0x0000_07E0 | limited to 0-31 | 0xC is Vines
];

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                               Pass Flags                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
impl PLCEntry {
    pub fn get_pass_object(&self) -> bool {
        self.codes[0] & 0x0000_4000 != 0
    }
    pub fn get_pass_camera(&self) -> bool {
        self.codes[0] & 0x0000_8000 != 0
    }
    pub fn get_pass_link(&self) -> bool {
        self.codes[0] & 0x0001_0000 != 0
    }
    pub fn get_pass_arrow(&self) -> bool {
        self.codes[0] & 0x0002_0000 != 0
    }
    pub fn get_pass_slingshot(&self) -> bool {
        self.codes[0] & 0x0004_0000 != 0
    }
    pub fn get_pass_beetle(&self) -> bool {
        self.codes[0] & 0x0008_0000 != 0
    }
    pub fn get_pass_clawshot(&self) -> bool {
        self.codes[0] & 0x0010_0000 != 0
    }
    pub fn get_pass_target(&self) -> bool {
        self.codes[0] & 0x0020_0000 != 0
    }
    pub fn get_pass_shadow(&self) -> bool {
        self.codes[0] & 0x0040_0000 != 0
    }
    pub fn get_pass_bomb(&self) -> bool {
        self.codes[0] & 0x0080_0000 != 0
    }
    pub fn get_pass_whip(&self) -> bool {
        self.codes[0] & 0x0100_0000 != 0
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                               Other Attributes                                                    //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl PLCEntry {}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                               Misc                                                                //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
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

impl PLCEntry {
    pub fn get_color(&self, filter_type: usize, range_selection: u32) -> Option<Vec4> {
        let val = ENTRY_FILTER.get(filter_type);
        if let Some(val) = val {
            match *val {
                Norm => return None,
                Range(ShiftMask {
                    code_idx,
                    shift,
                    mask,
                }) => {
                    let code = (self.codes[code_idx] >> shift) & mask;
                    if code == range_selection {
                        return Some(Vec4::ZERO.with_w(1.0f32).with_y(0.6f32));
                    }
                    return Some(Vec4::splat((code as f32) / (mask as f32)).with_w(1.0f32));
                }
                Single(ShiftMask {
                    code_idx,
                    shift,
                    mask,
                }) => {
                    let code = (self.codes[code_idx] >> shift) & mask;

                    if (code & 1) == 1 {
                        return Some(Vec4::new(0.0f32, 0.7f32, 0.0f32, 1.0f32));
                    } else {
                        return Some(Vec4::new(1f32, 1f32, 1f32, 1f32));
                    }
                }
            };
        }
        None
    }
}
