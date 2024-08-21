use binrw::{binrw, BinRead, NullString};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

#[binrw]
#[brw(big)]
#[derive(Debug)]
struct SizedString<const SIZE: usize> {
    #[br(temp)]
    #[bw(calc = self.str.as_bytes().try_into().unwrap())]
    __buffer: [u8; SIZE],
    #[br(calc = {
        if let Some(i) = __buffer.iter().position(|&a| a == b'\0') {
            String::from_utf8(__buffer[0..i].to_vec()).expect("Should be a valid string")
        } else {
            String::from_utf8(__buffer.to_vec()).expect("Should be a valid string")
        }
    })]
    #[bw(ignore)]
    str: String,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct FILE {
    unk: u16,
    dummy: u16,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct RMPLData {
    data: u16,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct RMPL {
    id: u8,
    #[br(temp)]
    #[bw(calc = (self.data.len() & 0xFF).try_into().unwrap())]
    count: u8,

    #[br(restore_position, count = count)]
    data: Vec<RMPLData>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct SCEN {
    name: SizedString<32>,
    room: u8,
    layer: u8,
    entrance: u8,
    night: u8,
    byte5: u8,
    flag6: u8,
    zero: u8,
    save_prompt: u8,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct CAM {
    unk1: [u8; 4],
    pos: [f32; 3],
    angle: f32,
    unk2: [u8; 8],
    name: SizedString<16>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct PCAM {
    pos1: [f32; 3],
    pos2: [f32; 3],
    angle: f32,
    unkf: f32,
    unk: [u8; 4],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct PATH {
    unk1: u8,
    unk2: u8,
    pnt_start_idx: u16,
    pnt_total_count: u16,
    unk3: [u8; 6],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct SPTH {
    unk1: u8,
    unk2: u8,
    pnt_start_idx: u16,
    pnt_total_count: u16,
    unk3: [u8; 6],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct PNT {
    pos: [f32; 3],
    unk: [u8; 4],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct SPNT {
    pos: [f32; 3],
    unk: [u8; 4],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct BPNT {
    pos1: [f32; 3],
    pos2: [f32; 3],
    pos3: [f32; 3],
    unk: [u8; 4],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct AREA {
    pos: [f32; 3],
    size: [f32; 3],
    angle: u16,
    area_link: i16,
    unk3: u8,
    _pad: [u8; 3],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct EVNT {
    unk1: [u8; 2],
    story_flag1: i16,
    story_flag2: i16,
    unk2: [u8; 3],
    exit_id: u8,
    unk3: [u8; 3],
    skipevent: u8,
    unk4: u8,
    sceneflag1: u8,
    sceneflag2: u8,
    skipflag: u8,
    dummy1: i16,
    item: i16,
    dummy2: i16,
    name: SizedString<32>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct PLY {
    storyflag: i16,
    play_cutscene: u8,
    byte4: u8,
    pos: [f32; 3],
    angle: [i16; 3],
    entrance_id: i16,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct STAG {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    size: [f32; 3],
    angle: [u16; 3],
    id: u16,
    name: SizedString<8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct LYSE {
    story_flag: i16,
    night: u8,
    layer: u8,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct STIF {
    wtf: [f32; 3],
    byte1: u8,
    flag_index: u8,
    byte3: u8,
    byte4: u8,
    unk1: [u8; 2],
    map_name_id: u8,
    unk2: u8,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct LYLT {
    layer: u8,
    demo_high: u8,
    demo_low: u8,
    dummy: u8,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct SOBJ {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    size: [f32; 3],
    angle: [i16; 3],
    id: u16,
    name: SizedString<8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct SOBS {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    size: [f32; 3],
    angle: [i16; 3],
    id: u16,
    name: SizedString<8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct OBJ {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    angle: [i16; 3],
    id: u16,
    name: SizedString<8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct OBJS {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    angle: [i16; 3],
    id: u16,
    name: SizedString<8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct STAS {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    size: [f32; 3],
    angle: [i16; 3],
    id: u16,
    name: SizedString<8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct SNDT {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    size: [f32; 3],
    angle: [i16; 3],
    id: u16,
    name: SizedString<8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct DOOR {
    params1: u32,
    params2: u32,
    pos: [f32; 3],
    angle: [i16; 3],
    id: u16,
    name: SizedString<8>,
}

#[derive(Debug)]
#[repr(C)]
pub struct OBJN {
    name: String,
}

#[derive(Debug)]
#[repr(C)]
pub struct ARCN {
    name: String,
}

#[derive(Debug, Default)]
pub struct LayerData {
    sobj: Vec<SOBJ>,
    sobs: Vec<SOBS>,
    obj_: Vec<OBJ>,
    objs: Vec<OBJS>,
    stag: Vec<STAG>,
    stas: Vec<STAS>,
    sndt: Vec<SNDT>,
    door: Vec<DOOR>,
    objn: Vec<OBJN>,
    arcn: Vec<ARCN>,
}

#[derive(Debug, Default)]
pub struct BZS {
    file: Vec<FILE>,
    stif: Vec<STIF>,
    rmpl: Vec<RMPL>,
    scen: Vec<SCEN>,
    cam_: Vec<CAM>,
    pcam: Vec<PCAM>,
    path: Vec<PATH>,
    spth: Vec<SPTH>,
    pnt_: Vec<PNT>,
    spnt: Vec<SPNT>,
    bpnt: Vec<BPNT>,
    area: Vec<AREA>,
    evnt: Vec<EVNT>,
    ply_: Vec<PLY>,
    lyse: Vec<LYSE>,
    lylt: Vec<LYLT>,
    sobj: Vec<SOBJ>,
    sobs: Vec<SOBS>,
    layers: [LayerData; 29],
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
struct Node {
    count: u16,  // Number of Entries in the Data
    _pad: u16,   // Padding
    offset: u32, // Data Offset
}

impl BZS {
    pub fn read<T: Clone + Read + Seek>(
        reader: &mut T,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Reading BZS Starts with a root node of V001
        let mut magic = [0u8; 4];

        reader.read_exact(&mut magic)?;

        if &magic != b"V001" {
            return Err("File has Invalid Magic".into());
        }
        let root = Node::read(reader)?;

        reader.seek(SeekFrom::Start(root.offset.into()))?;

        let mut bzs = Self::default();

        for _ in 0..root.count {
            // Create a Sub Reader to read the node data while keeping the main readers position
            let sub_reader = &mut reader.clone();

            // read the node magic
            reader.read_exact(&mut magic)?;
            let node = Node::read(reader)?;

            // Make sure sub_reader is used here as it now points to the data
            sub_reader.seek_relative(node.offset.into())?;
            for i in 0..node.count {
                match &magic {
                    b"FILE" => bzs.file.push(FILE::read(sub_reader)?),
                    b"RMPL" => bzs.rmpl.push(RMPL::read(sub_reader)?),
                    b"SCEN" => bzs.scen.push(SCEN::read(sub_reader)?),
                    b"CAM " => bzs.cam_.push(CAM::read(sub_reader)?),
                    b"PCAM" => bzs.pcam.push(PCAM::read(sub_reader)?),
                    b"PATH" => bzs.path.push(PATH::read(sub_reader)?),
                    b"SPTH" => bzs.spth.push(SPTH::read(sub_reader)?),
                    b"PNT " => bzs.pnt_.push(PNT::read(sub_reader)?),
                    b"SPNT" => bzs.spnt.push(SPNT::read(sub_reader)?),
                    b"BPNT" => bzs.bpnt.push(BPNT::read(sub_reader)?),
                    b"AREA" => bzs.area.push(AREA::read(sub_reader)?),
                    b"EVNT" => bzs.evnt.push(EVNT::read(sub_reader)?),
                    b"PLY " => bzs.ply_.push(PLY::read(sub_reader)?),
                    b"LYSE" => bzs.lyse.push(LYSE::read(sub_reader)?),
                    b"STIF" => bzs.stif.push(STIF::read(sub_reader)?),
                    b"SOBJ" => bzs.sobj.push(SOBJ::read(sub_reader)?),
                    b"SOBS" => bzs.sobs.push(SOBS::read(sub_reader)?),
                    b"LYLT" => bzs.lylt.push(LYLT::read(sub_reader)?),
                    b"LAY " => {
                        // Must Read 29 layers
                        let mut layer_mag = [0u8; 4];
                        if node.count != 29 {
                            return Err("\"LAY \": does not contain 29 entries".into());
                        }
                        let layer: &mut LayerData = bzs.layers.get_mut(i as usize).unwrap();

                        // Create a Sub Reader to read the node data while keeping the main readers position
                        let lay_reader = &mut sub_reader.clone();
                        let layer_node = Node::read(sub_reader)?;

                        // Make sure sub_reader is used here as it now points to the data
                        lay_reader.seek_relative(layer_node.offset.into())?;

                        for _ in 0..layer_node.count {
                            // Reading Layer Nodes now :)
                            let sub_reader = &mut lay_reader.clone();

                            // read the node magic
                            lay_reader.read_exact(&mut layer_mag)?;
                            let node = Node::read(lay_reader)?;

                            // Make sure sub_reader is used here as it now points to the data
                            sub_reader.seek_relative(node.offset.into())?;

                            // Used for ARCN/OBJN
                            let objn_arcn_pos: u64 = sub_reader.stream_position()?;

                            for _ in 0..node.count {
                                match &layer_mag {
                                    b"SOBJ" => layer.sobj.push(SOBJ::read(sub_reader)?),
                                    b"SOBS" => layer.sobs.push(SOBS::read(sub_reader)?),
                                    b"OBJ " => layer.obj_.push(OBJ::read(sub_reader)?),
                                    b"OBJS" => layer.objs.push(OBJS::read(sub_reader)?),
                                    b"STAS" => layer.stas.push(STAS::read(sub_reader)?),
                                    b"STAG" => layer.stag.push(STAG::read(sub_reader)?),
                                    b"SNDT" => layer.sndt.push(SNDT::read(sub_reader)?),
                                    b"DOOR" => layer.door.push(DOOR::read(sub_reader)?),
                                    b"OBJN" | b"ARCN" => {
                                        // u16 offset from the start of the first entry...
                                        let offset = sub_reader.read_u16::<BigEndian>()?;

                                        // Pos to return to after reading nullterm string
                                        let pos = sub_reader.stream_position()?;

                                        // read string
                                        sub_reader
                                            .seek(SeekFrom::Start(objn_arcn_pos + offset as u64))?;

                                        let string = NullString::read(sub_reader)?;

                                        // return sub_reader to pos
                                        sub_reader.seek(SeekFrom::Start(pos))?;

                                        // Insert into layer
                                        if &layer_mag == b"OBJN" {
                                            layer.objn.push(OBJN {
                                                name: string.to_string(),
                                            });
                                        } else {
                                            layer.arcn.push(ARCN {
                                                name: string.to_string(),
                                            });
                                        }
                                    }
                                    _ => println!("Unknown Magic: {}", unsafe {
                                        String::from_utf8_unchecked(magic.to_vec())
                                    }),
                                }
                            }
                        }
                    }
                    _ => println!("Unknown Magic: {}", unsafe {
                        String::from_utf8_unchecked(magic.to_vec())
                    }),
                }
            }
        }

        Ok(bzs)
    }
}
