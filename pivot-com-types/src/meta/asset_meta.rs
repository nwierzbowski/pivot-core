use iceoryx2::prelude::*;
use iceoryx2_bb_posix::shared_memory::*;

use crate::constants::MAX_NAME_LEN;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AssetMeta {
    // --- Offsets into mesh_shm_handle (The "Address Book") ---
    pub offset_uuids: u64,      //Points to [u8; 16][] in shm
    pub offset_verts: u64,      //Points to f32[] in shm
    pub offset_edges: u64,      //Points to u32[] in shm
    pub offset_vert_bases: u64, //Points to u32[] in shm (length = object_count + 1) stored cumulatively with final total
    pub offset_edge_bases: u64, //Points to u32[] in shm (length = object_count + 1) stored cumulatively with final total
    pub offset_transforms: u64, //Points to Transform[] in shm
    pub offset_object_names: u64,
    pub offset_object_name_lengths: u64,
    pub offset_group_name: u64, //Points to the group name string in shm

    // --- Totals ---
    pub object_count: u32, // Total objects in this group
    pub group_name_length: u16,
    pub surface_context: u16, // Id for surface context
}

impl AssetMeta {
    pub fn new(
        total_verts: u32,
        total_edges: u32,
        object_count: u32,
        surface_context: u16,
        group_name: &str,
        handle_name: &str,
    ) -> Result<(SharedMemory, Self), String> {
        // Helper to align the cursor to the next 8-byte boundary
        // This is a bitwise trick: (x + 7) & !7
        fn align_to_8(val: u64) -> u64 {
            (val + 7) & !7
        }

        let mut cursor = size_of::<Self>() as u64;

        let offset_uuids = cursor;
        cursor = align_to_8(offset_uuids + (object_count as u64 * 16));

        // 1. Vertices: [f32; total_verts * 3] -> 12 bytes per vertex
        let offset_verts = cursor;
        cursor = align_to_8(offset_verts + (total_verts as u64 * 12));

        // 2. Edges: [u32; total_edge_count * 2] -> 8 bytes per edge
        let offset_edges = cursor;
        cursor = align_to_8(offset_edges + (total_edges as u64 * 8));

        // 5. Transforms: [f32;  total_objects * 16] -> 64 bytes per object
        let offset_transforms = cursor;
        cursor = align_to_8(offset_transforms + (object_count as u64 * 64));

        // 6. Vert Bases: [u32; total_objects + 1] -> 4 bytes per entry (cumulative with final total)
        let offset_vert_bases = cursor;
        cursor = align_to_8(offset_vert_bases + ((object_count as u64 + 1) * 4));

        // 7. Edge Bases: [u32; total_objects + 1] -> 4 bytes per entry (cumulative with final total)
        let offset_edge_bases = cursor;
        cursor = align_to_8(offset_edge_bases + ((object_count as u64 + 1) * 4));

        let offset_object_names = cursor;
        cursor = align_to_8(offset_object_names + (object_count as u64 * MAX_NAME_LEN as u64));

        let offset_object_name_lengths = cursor;
        cursor = align_to_8(offset_object_name_lengths + (object_count as u64 * 2));

        let offset_group_name = cursor;
        cursor = align_to_8(offset_group_name + (group_name.len() as u64));

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
            offset_object_name_lengths,
            offset_group_name,

            object_count,
            group_name_length: group_name.len() as u16,
            surface_context,
        };

        let shm = group_metadata.create_shm_segment(&handle_name, total_size as usize)?;
        group_metadata.write_group_name(&shm, group_name);

        unsafe {
            // Get the base pointer of the newly created SHM
            let base_ptr = shm.base_address().as_ptr() as *mut Self;

            // Write the struct we just built into the very start of the SHM
            // This makes the SHM "Self-Describing"
            base_ptr.write(group_metadata.clone());
        }

        Ok((shm, group_metadata))
    }

    pub fn get_group_name<'a>(&self, shm_base: *const u8) -> &'a str {
        unsafe {
            let ptr = shm_base.add(self.offset_group_name as usize);
            let slice = std::slice::from_raw_parts(ptr, self.group_name_length as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }

    fn write_group_name(&self, shm: &SharedMemory, group_name: &str) {
        unsafe {
            let base_ptr = shm.base_address().as_ptr() as *mut u8;
            let ptr = base_ptr.add(self.offset_group_name as usize);
            std::ptr::copy_nonoverlapping(group_name.as_ptr(), ptr, group_name.len());
        }
    }

    fn create_shm_segment(&self, name: &str, size: usize) -> Result<SharedMemory, String> {
        let file_name = FileName::new(name.as_bytes())
            .map_err(|e| format!("invalid shared memory name '{}': {:?}", name, e))?;

        SharedMemoryBuilder::new(&file_name)
            .is_memory_locked(false)
            .creation_mode(CreationMode::PurgeAndCreate)
            .size(size)
            .permission(Permission::OWNER_ALL | Permission::GROUP_ALL)
            .zero_memory(true)
            .create()
            .map_err(|e| format!("failed to create shared memory '{}': {:?}", name, e))
    }
}