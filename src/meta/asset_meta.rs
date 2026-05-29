use crate::{constants::MAX_NAME_LEN, fields::Uuid};

/// Tuple type for asset data slices: (obj_uuids, verts, edges, loops, loop_bases, object_loop_counts, transforms, vert_counts, edge_counts, object_names, embeddings, canonical_transform, asset_embedding)
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
    *mut [u8],
    *mut [u8],
    *mut [u8],
);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Bounds3f {
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub min_y: f32,
    pub max_z: f32,
}

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
    pub offset_embeddings: usize, // Points to f32[256] in shm
    pub offset_canonical_transform: usize, // Points to f32[16] in shm
    pub offset_asset_embedding: usize, // Points to f32[256] in shm

    // --- Totals ---
    pub vert_count: u32,
    pub edge_count: u32,
    pub loop_count: u32,
    pub total_loop_lengths: u32,
    pub object_count: u32, // Total objects in this group
    pub group_name_length: u16,
    pub surface_context: u16, // Id for surface context
    pub uuid: Uuid,

    pub bounds: Bounds3f,
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

        // Embeddings: [f32; 256 * object_count] = 1024 * object_count bytes
        let offset_embeddings = cursor;
        cursor = align_to_32(offset_embeddings + object_count as usize * 256 * size_of::<f32>());

        // Canonical transform: [f32; 16] = 64 bytes
        let offset_canonical_transform = cursor;
        cursor = align_to_32(offset_canonical_transform + 16 * size_of::<f32>());

        // Asset embedding: [f32; 256] = 1024 bytes
        let offset_asset_embedding = cursor;
        cursor = align_to_32(offset_asset_embedding + 256 * size_of::<f32>());

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
            offset_embeddings,
            offset_canonical_transform,
            offset_asset_embedding,

            vert_count: total_verts,
            edge_count: total_edges,
            loop_count: total_loops,
            total_loop_lengths,
            object_count,
            group_name_length: group_name.len() as u16,
            surface_context,
            uuid,
            bounds: Bounds3f {
                min_x: 0f32,
                min_y: 0f32,
                min_z: 0f32,
                max_x: 0f32,
                max_y: 0f32,
                max_z: 0f32,
            },
        };

        Ok((group_metadata, total_size))
    }

    /// Returns all asset data slices as a tuple.
    /// The order is: (obj_uuids, verts, edges, loops, loop_bases, object_loop_counts, transforms, vert_counts, edge_counts, object_names, embeddings, canonical_transform, asset_embedding)
    pub fn get_slices(
        &self,
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
        *mut [u8],
        *mut [u8],
        *mut [u8],
    ) {
        let base_ptr = self as *const Self as *const u8 as *mut u8;
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
                from_raw_parts_mut(
                      base_ptr.add(self.offset_embeddings),
                      self.object_count as usize * 256 * size_of::<f32>(),
                  ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_canonical_transform),
                    16 * size_of::<f32>(),
                ),
                from_raw_parts_mut(
                    base_ptr.add(self.offset_asset_embedding),
                    256 * size_of::<f32>(),
                ),
             )
         }
     }

    /// Merge multiple assets into a single new asset.
    ///
    /// # Arguments
    /// * `dest_base` - Pointer to pre-allocated destination buffer
    /// * `sources` - Metadata pointer for the anchor asset and member assets
    ///
    /// Returns the new AssetMeta and total size.
    pub fn merge_assets(
        dest_base: *mut u8,
        anchor_meta_ptr: usize,
        member_meta_ptrs: &[usize],
    ) -> Result<(AssetMeta, usize), String> {
        use std::mem::size_of;

        // 1. Sum counts from all sources
        let mut total_verts = 0u32;
        let mut total_edges = 0u32;
        let mut total_loops = 0u32;
        let mut total_loop_lengths = 0u32;
        let mut object_count = 0u32;

        let anchor_meta = unsafe { &*(anchor_meta_ptr as *const AssetMeta) };

        // Add anchor's counts to totals
        total_verts += anchor_meta.vert_count;
        total_edges += anchor_meta.edge_count;
        total_loops += anchor_meta.loop_count;
        total_loop_lengths += anchor_meta.total_loop_lengths;
        object_count += anchor_meta.object_count;

        let mut surface_context = anchor_meta.surface_context;
        let mut uuid = anchor_meta.uuid;
        let mut group_name = anchor_meta.get_group_name();

        for &ptr in member_meta_ptrs {
            let meta = unsafe { &*(ptr as *const AssetMeta) };
            total_verts += meta.vert_count;
            total_edges += meta.edge_count;
            total_loops += meta.loop_count;
            total_loop_lengths += meta.total_loop_lengths;
            object_count += meta.object_count;
            surface_context = meta.surface_context;
            uuid = meta.uuid;
            group_name = meta.get_group_name();
        }

        // 2. Create new layout
        let (new_meta, total_size) = Self::new(
            total_verts, total_edges, total_loops,
            total_loop_lengths, object_count,
            surface_context, &group_name, uuid,
        )?;

  // 3. Copy data from sources — per-field concatenation
        unsafe {
            // Write the fully-initialized AssetMeta header to the top of the shared memory block
            std::ptr::write(dest_base as *mut AssetMeta, new_meta.clone());

            let dest = dest_base as *mut u8;

            // --- Object UUIDs (plain copy) ---
            {
                let mut cursor = new_meta.offset_uuids;
                {
                    let (src_uuids, ..) = anchor_meta.get_slices();
                    let size = uuids_byte_size(anchor_meta.object_count);
                    std::ptr::copy_nonoverlapping(src_uuids as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (src_uuids, ..) = src_meta.get_slices();
                    let size = uuids_byte_size(src_meta.object_count);
                    std::ptr::copy_nonoverlapping(src_uuids as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
            }

            // --- Vertices (plain copy) ---
            {
                let mut cursor = new_meta.offset_verts;
                {
                    let (_, src_verts, ..) = anchor_meta.get_slices();
                    let size = verts_byte_size(anchor_meta.vert_count);
                    std::ptr::copy_nonoverlapping(src_verts as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, src_verts, ..) = src_meta.get_slices();
                    let size = verts_byte_size(src_meta.vert_count);
                    std::ptr::copy_nonoverlapping(src_verts as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
            }

            // --- Edges (plain copy) ---
            {
                let mut cursor = new_meta.offset_edges;
                {
                    let (_, _, src_edges, ..) = anchor_meta.get_slices();
                    let size = edges_byte_size(anchor_meta.edge_count);
                    std::ptr::copy_nonoverlapping(src_edges as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, src_edges, ..) = src_meta.get_slices();
                    let size = edges_byte_size(src_meta.edge_count);
                    std::ptr::copy_nonoverlapping(src_edges as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
            }

            // --- Loop bases (cumulative index adjustment) ---
            {
                let mut cursor = new_meta.offset_loop_bases;
                let mut cum_offset = 0u32;
                {
                    let (_, _, _, _, src_loop_bases, ..) = anchor_meta.get_slices();
                    let size = loop_bases_byte_size(anchor_meta.loop_count);
                    let src_ptr = src_loop_bases as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=anchor_meta.loop_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                   cursor = (cursor + size + 31) & !31;
                    cum_offset += anchor_meta.total_loop_lengths;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, _, _, src_loop_bases, ..) = src_meta.get_slices();
                    let size = loop_bases_byte_size(src_meta.loop_count);
                    let src_ptr = src_loop_bases as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=src_meta.loop_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                    cursor = (cursor + size + 31) & !31;
                    cum_offset += src_meta.total_loop_lengths;
                }
            }

            // --- Loops (plain copy) ---
            {
                let mut cursor = new_meta.offset_loops;
                {
                    let (_, _, _, src_loops, ..) = anchor_meta.get_slices();
                    let size = loops_byte_size(anchor_meta.total_loop_lengths);
                    std::ptr::copy_nonoverlapping(src_loops as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, src_loops, ..) = src_meta.get_slices();
                    let size = loops_byte_size(src_meta.total_loop_lengths);
                    std::ptr::copy_nonoverlapping(src_loops as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
            }

            // --- Object loop counts (cumulative index adjustment) ---
            {
                let mut cursor = new_meta.offset_object_loop_counts;
                let mut cum_offset = 0u32;
                {
                    let (_, _, _, _, _, src_object_loop_counts, ..) = anchor_meta.get_slices();
                    let size = object_loop_counts_byte_size(anchor_meta.object_count);
                    let src_ptr = src_object_loop_counts as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=anchor_meta.object_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                    cursor = (cursor + size + 31) & !31;
                    cum_offset += anchor_meta.object_count;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, _, _, src_object_loop_counts, ..) = src_meta.get_slices();
                    let size = object_loop_counts_byte_size(src_meta.object_count);
                    let src_ptr = src_object_loop_counts as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=src_meta.object_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                    cursor = (cursor + size + 31) & !31;
                    cum_offset += src_meta.object_count;
                }
            }

            // --- Transforms (plain copy) ---
            {
                let mut cursor = new_meta.offset_transforms;
                {
                    let (_, _, _, _, _, _, src_transforms, ..) = anchor_meta.get_slices();
                    let size = transforms_byte_size(anchor_meta.object_count);
                    std::ptr::copy_nonoverlapping(src_transforms as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, _, _, _, src_transforms, ..) = src_meta.get_slices();
                    let size = transforms_byte_size(src_meta.object_count);
                    std::ptr::copy_nonoverlapping(src_transforms as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
            }

            // --- Vert bases (cumulative index adjustment) ---
            {
                let mut cursor = new_meta.offset_vert_bases;
                let mut cum_offset = 0u32;
                {
                    let (_, _, _, _, _, _, _, src_vert_bases, ..) = anchor_meta.get_slices();
                    let size = vert_counts_byte_size(anchor_meta.object_count);
                    let src_ptr = src_vert_bases as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=anchor_meta.object_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                    cursor = (cursor + size + 31) & !31;
                    cum_offset += anchor_meta.vert_count;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, _, _, _, _, src_vert_bases, ..) = src_meta.get_slices();
                    let size = vert_counts_byte_size(src_meta.object_count);
                    let src_ptr = src_vert_bases as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=src_meta.object_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                    cursor = (cursor + size + 31) & !31;
                    cum_offset += src_meta.vert_count;
                }
            }

            // --- Edge bases (cumulative index adjustment) ---
            {
                let mut cursor = new_meta.offset_edge_bases;
                let mut cum_offset = 0u32;
                {
                    let (_, _, _, _, _, _, _, _, src_edge_bases, ..) = anchor_meta.get_slices();
                    let size = edge_counts_byte_size(anchor_meta.object_count);
                    let src_ptr = src_edge_bases as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=anchor_meta.object_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                    cursor = (cursor + size + 31) & !31;
                    cum_offset += anchor_meta.edge_count;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, _, _, _, _, _, src_edge_bases, ..) = src_meta.get_slices();
                    let size = edge_counts_byte_size(src_meta.object_count);
                    let src_ptr = src_edge_bases as *const u32;
                    let dst_ptr = dest.add(cursor) as *mut u32;
                    for i in 0..=src_meta.object_count {
                        dst_ptr.add(i as usize).write(src_ptr.add(i as usize).read() + cum_offset);
                    }
                    cursor = (cursor + size + 31) & !31;
                    cum_offset += src_meta.edge_count;
                }
            }

            // --- Object names (plain copy) ---
            {
                let mut cursor = new_meta.offset_object_names;
                {
                    let (_, _, _, _, _, _, _, _, _, src_names, ..) = anchor_meta.get_slices();
                    let size = object_names_byte_size(anchor_meta.object_count);
                    std::ptr::copy_nonoverlapping(src_names as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, _, _, _, _, _, _, src_names, ..) = src_meta.get_slices();
                    let size = object_names_byte_size(src_meta.object_count);
                    std::ptr::copy_nonoverlapping(src_names as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
            }

            // --- Group name (first source only) ---
            {
                let first_src_meta = &*(anchor_meta_ptr as *const AssetMeta);
                let group_name = first_src_meta.get_group_name();
                let size = group_name.len();
                std::ptr::copy_nonoverlapping(
                    group_name.as_ptr(),
                    dest.add(new_meta.offset_group_name),
                    size,
                );
            }

            // --- Embeddings (plain copy) ---
            {
                let mut cursor = new_meta.offset_embeddings;
                {
                    let (_, _, _, _, _, _, _, _, _, _, src_embeddings, ..) = anchor_meta.get_slices();
                    let size = anchor_meta.object_count as usize * 256 * size_of::<f32>();
                    std::ptr::copy_nonoverlapping(src_embeddings as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
                for &ptr in member_meta_ptrs {
                    let src_meta = &*(ptr as *const AssetMeta);
                    let (_, _, _, _, _, _, _, _, _, _, src_embeddings, ..) = src_meta.get_slices();
                    let size = src_meta.object_count as usize * 256 * size_of::<f32>();
                    std::ptr::copy_nonoverlapping(src_embeddings as *const u8, dest.add(cursor), size);
                    cursor = (cursor + size + 31) & !31;
                }
            }

            // --- Canonical transform (first source only) ---
            {
                let first_src_meta = &*(anchor_meta_ptr as *const AssetMeta);
                let (_, _, _, _, _, _, _, _, _, _, _, src_canonical_transform, _) = first_src_meta.get_slices();
                let size = 16 * size_of::<f32>();
                std::ptr::copy_nonoverlapping(
                    src_canonical_transform as *const u8,
                    dest.add(new_meta.offset_canonical_transform),
                    size,
                );
            }

            // --- Asset embedding (first source only) ---
            {
                let first_src_meta = &*(anchor_meta_ptr as *const AssetMeta);
                let (_, _, _, _, _, _, _, _, _, _, _, _, src_asset_embedding) = first_src_meta.get_slices();
                let size = 256 * size_of::<f32>();
                std::ptr::copy_nonoverlapping(
                    src_asset_embedding as *const u8,
                    dest.add(new_meta.offset_asset_embedding),
                    size,
                );
            }

            Ok((new_meta, total_size))
        }
    }

    /// Get the group name string from shared memory.
    pub fn get_group_name(&self) -> String {
        unsafe {
            let base = self as *const Self as usize;
            let name_ptr = (base + self.offset_group_name) as *const u8;
            let len = self.group_name_length as usize;
            let name_bytes = std::slice::from_raw_parts(name_ptr, len);
            std::str::from_utf8(name_bytes)
                .unwrap_or("")
                .trim_end_matches('\0')
                .to_string()
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
    (total_loops + 1) as usize * size_of::<u32>()
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
