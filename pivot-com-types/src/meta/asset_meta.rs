use std::ptr::{NonNull, addr_of_mut};

use iceoryx2::prelude::*;
use iceoryx2_bb_posix::shared_memory::*;

use crate::{
    constants::MAX_NAME_LEN,
    fields::{Edge, Matrix4x4, Uuid, Vert},
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AssetMeta {
    // --- Offsets into mesh_shm_handle (The "Address Book") ---
    pub offset_uuids: usize,      //Points to [u8; 16][] in shm
    pub offset_verts: usize,      //Points to f32[] in shm
    pub offset_edges: usize,      //Points to u32[] in shm
    pub offset_vert_bases: usize, //Points to u32[] in shm (length = object_count + 1) stored cumulatively with final total
    pub offset_edge_bases: usize, //Points to u32[] in shm (length = object_count + 1) stored cumulatively with final total
    pub offset_transforms: usize, //Points to Transform[] in shm
    pub offset_object_names: usize,
    pub offset_group_name: usize, //Points to the group name string in shm

    // --- Totals ---
    pub vert_count: u32,
    pub edge_count: u32,
    pub object_count: u32, // Total objects in this group
    pub group_name_length: u16,
    pub surface_context: u16, // Id for surface context
    pub uuid: Uuid,
}

impl AssetMeta {
    pub fn new(
        total_verts: u32,
        total_edges: u32,
        object_count: u32,
        surface_context: u16,
        group_name: &str,
        uuid: Uuid,
    ) -> Result<(AssetMeta, usize), String> {
        // Helper to align the cursor to the next 8-byte boundary
        // This is a bitwise trick: (x + 7) & !7
        fn align_to_8(val: usize) -> usize {
            (val + 7) & !7
        }

        let count = object_count as usize;

        let mut cursor = size_of::<Self>() as usize;

        let offset_uuids = cursor;
        cursor = align_to_8(offset_uuids + (count * size_of::<Uuid>()));

        // 1. Vertices: [f32; total_verts * 3] -> 12 bytes per vertex
        let offset_verts = cursor;
        cursor = align_to_8(offset_verts + (total_verts as usize * size_of::<Vert>()));

        // 2. Edges: [u32; total_edge_count * 2] -> 8 bytes per edge
        let offset_edges = cursor;
        cursor = align_to_8(offset_edges + (total_edges as usize * size_of::<Edge>()));
        // 5. Transforms: [f32;  total_objects * 16] -> 64 bytes per object
        let offset_transforms = cursor;
        cursor = align_to_8(offset_transforms + (count * size_of::<Matrix4x4>()));

        // 6. Vert Bases: [u32; total_objects + 1] -> 4 bytes per entry (cumulative with final total)
        let offset_vert_bases = cursor;
        cursor = align_to_8(offset_vert_bases + ((count + 1) * size_of::<u32>()));

        // 7. Edge Bases: [u32; total_objects + 1] -> 4 bytes per entry (cumulative with final total)
        let offset_edge_bases = cursor;
        cursor = align_to_8(offset_edge_bases + ((count + 1) * size_of::<u32>()));

        let offset_object_names = cursor;
        cursor = align_to_8(offset_object_names + (count * MAX_NAME_LEN));

        let offset_group_name = cursor;
        cursor = align_to_8(offset_group_name + (group_name.len()));

        // The final cursor value is the total bytes needed for the SHM segment
        let total_size = cursor;

        // 8. Construct the GroupFull "Blueprint"
        let group_metadata = self::AssetMeta {
            offset_uuids,
            offset_verts,
            offset_edges,
            offset_vert_bases,
            offset_edge_bases,
            offset_transforms,
            offset_object_names,
            offset_group_name,

            vert_count: total_verts,
            edge_count: total_edges,
            object_count,
            group_name_length: group_name.len() as u16,
            surface_context,
            uuid,
        };

        // let shm = group_metadata.create_shm_segment(&handle_name, total_size as usize)?;
        // group_metadata.write_group_name(&shm, group_name);

        // unsafe {
        //     // Get the base pointer of the newly created SHM
        //     let base_ptr = shm.base_address().as_ptr() as *mut Self;

        //     // Write the struct we just built into the very start of the SHM
        //     // This makes the SHM "Self-Describing"
        //     base_ptr.write(group_metadata.clone());
        // }

        Ok((group_metadata, total_size))
    }

    pub fn get_group_name<'a>(&self, shm_base: *const u8) -> &'a str {
        unsafe {
            let ptr = shm_base.add(self.offset_group_name as usize);
            let slice = std::slice::from_raw_parts(ptr, self.group_name_length as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }

    pub fn write_group_name(&mut self, group_name: &str) {
        unsafe {
            let ptr = addr_of_mut!(*self).add(self.offset_group_name as usize) as *mut u8;

            std::ptr::copy_nonoverlapping(group_name.as_ptr(), ptr, group_name.len());
        }
    }
}
