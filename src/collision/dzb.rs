use byteorder::{ReadBytesExt, BE};
use std::io::{Read, Seek, SeekFrom};

use glam::Vec3;

struct Header {
    vert_count: u32,
    vert_offset: u32,
    triangle_count: u32,
    triangle_offset: u32,
    blocks_count: u32,
    blocks_offset: u32,
    tree_nodes_count: u32,
    tree_nodes_offset: u32,
    groups_count: u32,
    groups_offset: u32,
    properties_count: u32,
    properties_offset: u32,
    padding: u32,
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vert_idx: [u16; 3],
    pub prop_idx: u16,
    pub group_idx: u16,
}

#[derive(Debug, Clone)]
pub struct OctreeNode {
    pub flags: u16,
    pub parent_node_idx: u16,
    pub branches: [u16; 8],
}

#[derive(Debug, Clone)]
pub struct Block {
    pub starting_tri_idx: u16,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub name_offset: u32,
    pub scale: glam::Vec3,
    pub rotation: [i16; 3],
    pub unk1: u16,
    pub translation: glam::Vec3,
    pub parent_group_idx: u16,
    pub next_sibling_group: u16,
    pub first_child_group_index: u16,
    pub room_id: u16,
    pub first_vtx_idx: u16,
    pub tree_index: u16,
    pub info: u16,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub info1: u32,
    pub info2: u32,
    pub info3: u32,
    pub pass_flag: u32,
}

#[derive(Debug, Clone)]
pub struct DZB {
    pub verts: Vec<Vec3>,
    pub tris: Vec<Triangle>,
    pub blocks: Vec<Block>,
    pub tree_nodes: Vec<OctreeNode>,
    pub groups: Vec<Group>,
    pub properties: Vec<Property>,
}

impl DZB {
    pub fn from_file<R: Seek + Read>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Read Header                                        //
        ///////////////////////////////////////////////////////////////////////////////////////////
        let header = Header {
            vert_count: reader.read_u32::<BE>()?,
            vert_offset: reader.read_u32::<BE>()?,
            triangle_count: reader.read_u32::<BE>()?,
            triangle_offset: reader.read_u32::<BE>()?,
            blocks_count: reader.read_u32::<BE>()?,
            blocks_offset: reader.read_u32::<BE>()?,
            tree_nodes_count: reader.read_u32::<BE>()?,
            tree_nodes_offset: reader.read_u32::<BE>()?,
            groups_count: reader.read_u32::<BE>()?,
            groups_offset: reader.read_u32::<BE>()?,
            properties_count: reader.read_u32::<BE>()?,
            properties_offset: reader.read_u32::<BE>()?,
            padding: reader.read_u32::<BE>()?,
        };

        let mut verts: Vec<Vec3> = Vec::with_capacity(header.vert_count as _);
        let mut tris: Vec<Triangle> = Vec::with_capacity(header.triangle_count as _);
        let mut blocks: Vec<Block> = Vec::with_capacity(header.blocks_count as _);
        let mut tree_nodes: Vec<OctreeNode> = Vec::with_capacity(header.tree_nodes_count as _);
        let mut groups: Vec<Group> = Vec::with_capacity(header.groups_count as _);
        let mut properties: Vec<Property> = Vec::with_capacity(header.properties_count as _);

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                               Read Vectored Data                                      //
        ///////////////////////////////////////////////////////////////////////////////////////////

        // Vertices
        reader
            .seek(SeekFrom::Start(header.vert_offset as _))
            .expect("Should be in bounds");
        for _ in 0..header.vert_count {
            verts.push(Vec3::new(
                reader.read_f32::<BE>()?,
                reader.read_f32::<BE>()?,
                reader.read_f32::<BE>()?,
            ));
        }

        // Triangles
        reader
            .seek(SeekFrom::Start(header.triangle_offset as _))
            .expect("Should be in bounds");
        for _ in 0..header.triangle_count {
            tris.push(Triangle {
                vert_idx: [
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                ],
                prop_idx: reader.read_u16::<BE>()?,
                group_idx: reader.read_u16::<BE>()?,
            });
        }

        // Blocks
        reader
            .seek(SeekFrom::Start(header.blocks_offset as _))
            .expect("Should be in bounds");
        for _ in 0..header.blocks_count {
            blocks.push(Block {
                starting_tri_idx: reader.read_u16::<BE>()?,
            });
        }

        // Tree Nodes
        reader
            .seek(SeekFrom::Start(header.tree_nodes_offset as _))
            .expect("Should be in bounds");
        for _ in 0..header.tree_nodes_count {
            tree_nodes.push(OctreeNode {
                flags: reader.read_u16::<BE>()?,
                parent_node_idx: reader.read_u16::<BE>()?,
                branches: [
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                    reader.read_u16::<BE>()?,
                ],
            });
        }

        // Groups
        reader
            .seek(SeekFrom::Start(header.groups_offset as _))
            .expect("Should be in bounds");
        for _ in 0..header.groups_count {
            groups.push(Group {
                name_offset: reader.read_u32::<BE>()?,
                scale: Vec3::new(
                    reader.read_f32::<BE>()?,
                    reader.read_f32::<BE>()?,
                    reader.read_f32::<BE>()?,
                ),
                rotation: [
                    reader.read_i16::<BE>()?,
                    reader.read_i16::<BE>()?,
                    reader.read_i16::<BE>()?,
                ],
                unk1: reader.read_u16::<BE>()?,
                translation: Vec3::new(
                    reader.read_f32::<BE>()?,
                    reader.read_f32::<BE>()?,
                    reader.read_f32::<BE>()?,
                ),
                parent_group_idx: reader.read_u16::<BE>()?,
                next_sibling_group: reader.read_u16::<BE>()?,
                first_child_group_index: reader.read_u16::<BE>()?,
                room_id: reader.read_u16::<BE>()?,
                first_vtx_idx: reader.read_u16::<BE>()?,
                tree_index: reader.read_u16::<BE>()?,
                info: reader.read_u16::<BE>()?,
            });
        }

        // Properties
        reader
            .seek(SeekFrom::Start(header.properties_offset as _))
            .expect("Should be in bounds");
        for _ in 0..header.properties_count {
            properties.push(Property {
                info1: reader.read_u32::<BE>()?,
                info2: reader.read_u32::<BE>()?,
                info3: reader.read_u32::<BE>()?,
                pass_flag: reader.read_u32::<BE>()?,
            });
        }

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Build Model                                        //
        ///////////////////////////////////////////////////////////////////////////////////////////

        Ok(Self {
            verts,
            tris,
            blocks,
            tree_nodes,
            groups,
            properties,
        })
    }
}
