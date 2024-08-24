use byteorder::{ReadBytesExt, BE};
use std::{
    cmp::max,
    io::{Read, Seek, SeekFrom},
};

use glam::Vec3;

struct Header {
    pos_dat_offset: u32,
    nrm_data_offset: u32,
    prism_data_offset: u32,
    block_data_offset: u32,
    prism_thickness: f32,
    area_min_pos: Vec3,
    area_x_width_mask: u32,
    area_y_width_mask: u32,
    area_z_width_mask: u32,
    block_width_shift: u32,
    area_x_blocks_shift: u32,
    area_xy_blocks_shift: u32,
}

#[derive(Debug, Clone)]
pub enum Octree {
    Leaf(Vec<u16>),
    Branch(Vec<Octree>),
}

impl Octree {
    fn new<R: Seek + Read>(value: u32, reader: &mut R, num_children: u32) -> Self {
        if (value & 0x8000_0000) != 0 {
            // First entry is considered the last of the previous
            let offset: u64 = ((value & 0x7FFF_FFFF) + size_of::<u16>() as u32).into();
            reader
                .seek(SeekFrom::Start(offset))
                .expect("reader should be Inbounds");

            let mut indices = Vec::<u16>::new();

            while let Ok(idx) = reader.read_u16::<BE>() {
                if idx == 0 {
                    break;
                }
                indices.push(idx);
            }
            Self::Leaf(indices)
        } else {
            let mut children = Vec::new();
            // Loop Through all children and add it
            for i in 0..num_children {
                reader
                    .seek(SeekFrom::Start((i * 4 + value) as _))
                    .expect("Should be in bounds");
                let new_val: u32 = reader.read_u32::<BE>().expect("Should be Inbounds");
                children.push(Octree::new(new_val + value, reader, 8));
            }
            Self::Branch(children)
        }
    }

    fn get_highest_index(&self) -> usize {
        match self {
            Octree::Leaf(vals) => {
                let mut max = 0;
                for &val in vals {
                    if val > max {
                        max = val;
                    }
                }
                max as usize
            }
            Octree::Branch(children) => {
                let mut max = 0;
                for child in children {
                    let val = child.get_highest_index();
                    if val > max {
                        max = val;
                    }
                }
                max as usize
            }
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Prism {
    height: f32,
    pos_i: u16,
    fnrm_i: u16,
    enrm_i: [u16; 3],
    attribute: u16,
}

#[derive(Debug, Clone)]
pub struct KCLTriangle {
    pub vertices: [Vec3; 3],
    // pub edge_normals: [Vec3; 3],
    pub face_normal: Vec3,
    pub attribute: u16,
}

impl Prism {
    pub fn from_buffer<R: Seek + Read>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            height: reader.read_f32::<BE>()?,
            pos_i: reader.read_u16::<BE>()?,
            fnrm_i: reader.read_u16::<BE>()?,
            enrm_i: [
                reader.read_u16::<BE>()?,
                reader.read_u16::<BE>()?,
                reader.read_u16::<BE>()?,
            ],
            attribute: reader.read_u16::<BE>()?,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KCL {
    pub vtx: Vec<Vec3>,
    pub nrm: Vec<Vec3>,
    pub prism: Vec<Prism>,
    pub octree: Octree,
    pub prism_thickness: f32,
    pub area_min_pos: Vec3,
}

impl KCL {
    pub fn from_file<R: Seek + Read>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Read Header                                        //
        ///////////////////////////////////////////////////////////////////////////////////////////
        let header = Header {
            pos_dat_offset: reader.read_u32::<BE>()?,
            nrm_data_offset: reader.read_u32::<BE>()?,
            prism_data_offset: reader.read_u32::<BE>()?,
            block_data_offset: reader.read_u32::<BE>()?,
            prism_thickness: reader.read_f32::<BE>()?,
            area_min_pos: Vec3::new(
                reader.read_f32::<BE>()?, // -20000
                reader.read_f32::<BE>()?, // -15000
                reader.read_f32::<BE>()?, // -26315
            ),
            area_x_width_mask: reader.read_u32::<BE>()?, // FFFF0000
            area_y_width_mask: reader.read_u32::<BE>()?, // FFFF8000
            area_z_width_mask: reader.read_u32::<BE>()?, // FFFF0000
            block_width_shift: reader.read_u32::<BE>()?, // 14
            area_x_blocks_shift: reader.read_u32::<BE>()?, // 2
            area_xy_blocks_shift: reader.read_u32::<BE>()?, // 3
        };

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                               Read Spatial Index                                      //
        ///////////////////////////////////////////////////////////////////////////////////////////

        // Parse Octree
        let shift = header.block_width_shift;
        let num_initial = ((!header.area_z_width_mask >> shift) << header.area_xy_blocks_shift)
            | ((!header.area_y_width_mask >> shift) << header.area_x_blocks_shift)
            | (!header.area_x_width_mask >> shift);

        let octree: Octree = Octree::new(header.block_data_offset, reader, num_initial);

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Read Prisms                                        //
        ///////////////////////////////////////////////////////////////////////////////////////////

        let num_prisms = octree.get_highest_index();
        let mut raw_prisms = Vec::with_capacity(num_prisms);

        // Seek to the offset provided by the header + 0x10 (size of raw prism since its 1 indexed)
        let _ = reader
            .seek(SeekFrom::Start(
                (header.prism_data_offset as usize + std::mem::size_of::<Prism>()) as _,
            ))
            .expect("Should be in bounds");

        // Add all prisms
        for _ in 0..num_prisms {
            raw_prisms.push(Prism::from_buffer(reader)?);
        }

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                               Read Position Vectors                                   //
        ///////////////////////////////////////////////////////////////////////////////////////////

        // Get the highest position index from the prisms
        let mut num_pos = 0;
        for prism in &raw_prisms {
            num_pos = max(num_pos, prism.pos_i);
        }
        num_pos += 1;

        // Allocate space
        let mut position_vecs = Vec::with_capacity(num_pos as usize);

        // Seek and Read
        reader
            .seek(SeekFrom::Start(header.pos_dat_offset as _))
            .expect("Should be in bounds");
        for _ in 0..num_pos {
            position_vecs.push(Vec3::new(
                reader.read_f32::<BE>()?,
                reader.read_f32::<BE>()?,
                reader.read_f32::<BE>()?,
            ));
        }

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                               Read Normal Vectors                                     //
        ///////////////////////////////////////////////////////////////////////////////////////////

        // Get the highest position index from the prisms
        let mut num_norms = 0;
        for prism in &raw_prisms {
            num_norms = max(num_norms, prism.fnrm_i);
            num_norms = max(num_norms, prism.enrm_i[0]);
            num_norms = max(num_norms, prism.enrm_i[1]);
            num_norms = max(num_norms, prism.enrm_i[2]);
        }
        num_norms += 1;

        // Allocate space
        let mut normal_vecs = Vec::with_capacity(num_norms as usize);

        // Seek and Read
        reader
            .seek(SeekFrom::Start(header.nrm_data_offset as _))
            .expect("Should be in bounds");
        for _ in 0..num_norms {
            normal_vecs.push(Vec3::new(
                reader.read_f32::<BE>()?,
                reader.read_f32::<BE>()?,
                reader.read_f32::<BE>()?,
            ));
        }

        Ok(Self {
            vtx: position_vecs,
            nrm: normal_vecs,
            prism: raw_prisms,
            octree: octree,
            prism_thickness: header.prism_thickness,
            area_min_pos: header.area_min_pos,
        })
    }

    pub fn get_triangles(&self) -> Vec<KCLTriangle> {
        let mut tris = Vec::with_capacity(self.prism.len());
        for prism in &self.prism {
            let pos: Vec3 = self.vtx[prism.pos_i as usize];
            let fnrm: Vec3 = self.nrm[prism.fnrm_i as usize];
            let enrm1: Vec3 = self.nrm[prism.enrm_i[0] as usize];
            let enrm2: Vec3 = self.nrm[prism.enrm_i[1] as usize];
            let enrm3: Vec3 = self.nrm[prism.enrm_i[2] as usize];

            let cross_a = fnrm.cross(enrm1);
            let cross_b = fnrm.cross(enrm2);

            tris.push(KCLTriangle {
                vertices: [
                    pos,
                    pos + cross_b * (prism.height / cross_b.dot(enrm3)),
                    pos + cross_a * (prism.height / cross_a.dot(enrm3)),
                ],
                face_normal: fnrm,
                // edge_normals: [enrm1, enrm2, enrm3],
                attribute: prism.attribute,
            })
        }
        tris
    }
}
