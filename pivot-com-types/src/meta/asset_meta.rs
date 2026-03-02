use crate::{constants::MAX_NAME_LEN, fields::Uuid};

/// Tuple type for asset data slices: (obj_uuids, verts, edges, loops, loop_bases, object_loop_counts, transforms, vert_counts, edge_counts, object_names)
pub type AssetDataSlices = (
    *mut [u8],
    *mut [u8],
    *mut [u8],
    *mut [u8],
    *mut [u8],
    *mut [u8],
    *mut [u8],
    *mut [u8],
    *mut [u8],
    *mut [u8],
);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AssetMeta {
    // --- Offsets into mesh_shm_handle (The "Address Book") ---
    pub offset_uuids: usize, //Points to [u8; 16][] in shm
    pub offset_verts: usize, //Points to f32[] in shm
    pub offset_edges: usize, //Points to u32[] in shm
    ///Entry per face corner, stores vert index
    pub offset_loops: usize,
    ///Entry per face, stores each face first index into loops with total at entry n
    pub offset_loop_bases: usize,
    ///Entry per object, stores index into loop lengths with total at entry n
    pub offset_object_loop_counts: usize,
    pub offset_vert_bases: usize, //Points to u32[] in shm (length = object_count + 1) stored cumulatively with final total
    pub offset_edge_bases: usize, //Points to u32[] in shm (length = object_count + 1) stored cumulatively with final total
    pub offset_transforms: usize, //Points to Transform[] in shm
    pub offset_object_names: usize,
    pub offset_group_name: usize, //Points to the group name string in shm

    // --- Totals ---
    pub vert_count: u32,
    pub edge_count: u32,
    pub loop_count: u32,
    pub total_loop_lengths: u32,
    pub object_count: u32, // Total objects in this group
    pub group_name_length: u16,
    pub surface_context: u16, // Id for surface context
    pub uuid: Uuid,
}

impl AssetMeta {
    pub fn new(
        total_verts: u32,
        total_edges: u32,
        total_loops: u32,
        total_loop_lengths: u32,
        object_count: u32,
        surface_context: u16,
        group_name: &str,
        uuid: Uuid,
    ) -> Result<(AssetMeta, usize), String> {
        // Helper to align the cursor to the next 32-byte boundary
        // Required for AVX SIMD stability and performance.
        fn align_to_32(val: usize) -> usize {
            (val + 31) & !31
        }

        let mut cursor = size_of::<Self>() as usize;

        let offset_uuids = cursor;
        cursor = align_to_32(offset_uuids + uuids_byte_size(object_count));

        // 1. Vertices: [f32; total_verts * 3] -> 12 bytes per vertex
        let offset_verts = cursor;
        cursor = align_to_32(offset_verts + verts_byte_size(total_verts));

        // 2. Edges: [u32; total_edge_count * 2] -> 8 bytes per edge
        let offset_edges = cursor;
        cursor = align_to_32(offset_edges + edges_byte_size(total_edges));

        let offset_loop_bases = cursor;
        cursor = align_to_32(offset_loop_bases + loop_bases_byte_size(total_loops));

        let offset_loops = cursor;
        cursor = align_to_32(offset_loops + loops_byte_size(total_loop_lengths));

        let offset_object_loop_counts = cursor;
        cursor =
            align_to_32(offset_object_loop_counts + object_loop_counts_byte_size(object_count));

        // 5. Transforms: [f32;  total_objects * 16] -> 64 bytes per object
        let offset_transforms = cursor;
        cursor = align_to_32(offset_transforms + transforms_byte_size(object_count));

        // 6. Vert Bases: [u32; total_objects + 1] -> 4 bytes per entry (cumulative with final total)
        let offset_vert_bases = cursor;
        cursor = align_to_32(offset_vert_bases + vert_counts_byte_size(object_count));

        // 7. Edge Bases: [u32; total_objects + 1] -> 4 bytes per entry (cumulative with final total)
        let offset_edge_bases = cursor;
        cursor = align_to_32(offset_edge_bases + edge_counts_byte_size(object_count));

        let offset_object_names = cursor;
        cursor = align_to_32(offset_object_names + object_names_byte_size(object_count));

        let offset_group_name = cursor;
        cursor = align_to_32(offset_group_name + (group_name.len()));

        // The final cursor value is the total bytes needed for the SHM segment
        let total_size = cursor;

        // 8. Construct the GroupFull "Blueprint"
        let group_metadata = self::AssetMeta {
            offset_uuids,
            offset_verts,
            offset_edges,
            offset_loop_bases,
            offset_loops,
            offset_object_loop_counts,
            offset_vert_bases,
            offset_edge_bases,
            offset_transforms,
            offset_object_names,
            offset_group_name,

            vert_count: total_verts,
            edge_count: total_edges,
            loop_count: total_loops,
            total_loop_lengths,
            object_count,
            group_name_length: group_name.len() as u16,
            surface_context,
            uuid,
        };

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
            // 1. Get the base address of THIS struct instance in SHM
            let base_ptr = self as *mut Self as *mut u8;

            // 2. Jump forward by the offset stored IN the struct
            let dest_ptr = base_ptr.add(self.offset_group_name as usize);

            // 3. Perform the raw copy
            std::ptr::copy_nonoverlapping(group_name.as_ptr(), dest_ptr, group_name.len());
        }
    }

    /// Returns all asset data slices as a tuple.
    /// The order is: (obj_uuids, verts, edges, loops, loop_bases, object_loop_counts, transforms, vert_counts, edge_counts, object_names)
    pub fn get_slices(
        &mut self,
    ) -> (
        *mut [u8],
        *mut [u8],
        *mut [u8],
        *mut [u8],
        *mut [u8],
        *mut [u8],
        *mut [u8],
        *mut [u8],
        *mut [u8],
        *mut [u8],
    ) {
        let base_ptr = self as *mut Self as *mut u8;
        use std::slice::from_raw_parts_mut;

        unsafe {
            (
                from_raw_parts_mut(
                    base_ptr.add(self.offset_uuids),
                    uuids_byte_size(self.object_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_verts),
                    verts_byte_size(self.vert_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_edges),
                    edges_byte_size(self.edge_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_loops),
                    loops_byte_size(self.total_loop_lengths),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_loop_bases),
                    loop_bases_byte_size(self.loop_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_object_loop_counts),
                    object_loop_counts_byte_size(self.object_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_transforms),
                    transforms_byte_size(self.object_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_vert_bases),
                    vert_counts_byte_size(self.object_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_edge_bases),
                    edge_counts_byte_size(self.object_count),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_object_names),
                    object_names_byte_size(self.object_count),
                ),
            )
        }
    }
}

// Private helper functions for calculating byte sizes
fn verts_byte_size(total_verts: u32) -> usize {
    total_verts as usize * (3 * size_of::<f32>())
}

fn edges_byte_size(total_edges: u32) -> usize {
    total_edges as usize * (2 * size_of::<u32>())
}

fn object_loop_counts_byte_size(object_count: u32) -> usize {
    (object_count + 1) as usize * size_of::<u32>()
}

fn loop_bases_byte_size(total_loops: u32) -> usize {
    total_loops as usize * size_of::<u32>()
}

fn loops_byte_size(total_loop_lengths: u32) -> usize {
    total_loop_lengths as usize * (size_of::<u32>())
}

fn transforms_byte_size(object_count: u32) -> usize {
    object_count as usize * (16 * size_of::<f32>())
}

fn vert_counts_byte_size(object_count: u32) -> usize {
    (object_count + 1) as usize * size_of::<u32>()
}

fn edge_counts_byte_size(object_count: u32) -> usize {
    (object_count + 1) as usize * size_of::<u32>()
}

fn object_names_byte_size(object_count: u32) -> usize {
    object_count as usize * MAX_NAME_LEN
}

fn uuids_byte_size(object_count: u32) -> usize {
    object_count as usize * size_of::<Uuid>()
}
