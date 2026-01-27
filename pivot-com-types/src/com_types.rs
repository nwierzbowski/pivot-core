use iceoryx2::prelude::*;
use iceoryx2_bb_posix::shared_memory::*;

pub const OP_STANDARDIZE_GROUPS: u16 = 1;
pub const OP_STANDARDIZE_SYNCED_GROUPS: u16 = 2;
pub const OP_SET_SURFACE_TYPES: u16 = 3;
pub const OP_DROP_GROUPS: u16 = 4;
pub const OP_ORGANIZE_OBJECTS: u16 = 5;
pub const OP_GET_SURFACE_TYPES: u16 = 6;
pub const OP_STOP_ENGINE: u16 = 7;
// Fixed-size strings are essential for Zero-Copy structs
pub const MAX_NAME_LEN: usize = 64;
pub const MAX_HANDLE_LEN: usize = 32; // For OS SHM paths
pub const MAX_INLINE_DATA: usize = 65536; // 64 KB (L1 Cache Friendly)

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct GroupNames {
    pub group_name: [u8; MAX_NAME_LEN],
}

impl GroupNames {
    pub fn new(group_name: &str) -> Self {
        let mut gn = GroupNames {
            group_name: [0; MAX_NAME_LEN],
        };
        let bytes = group_name.as_bytes();
        let len = bytes.len().min(MAX_NAME_LEN - 1);
        gn.group_name[..len].copy_from_slice(&bytes[..len]);
        gn
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct GroupSurface {
    pub group_name: [u8; MAX_NAME_LEN],
    pub surface_type: u64, // "wood", "metal", etc.
}

impl GroupSurface {
    pub fn new(group_name: &str, surface_type: u64) -> Self {
        let mut surf = GroupSurface {
            group_name: [0; MAX_NAME_LEN],
            surface_type,
        };

        surf.set_group_name(group_name);
        surf
    }

    fn set_group_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(MAX_NAME_LEN - 1);
        self.group_name[..len].copy_from_slice(&bytes[..len]);
    }
}

#[repr(C)]
pub struct ShmOffset {
    pub meta_data_offset: u64,
    pub mesh_shm_handle: [u8; MAX_HANDLE_LEN], // The single SHM containing ALL data for this group
}

impl ShmOffset {
    pub fn new(meta_data_offset: u64, mesh_shm_handle: String) -> Self {
        let mut handle_bytes = [0u8; MAX_HANDLE_LEN];
        let bytes = mesh_shm_handle.as_bytes();
        let len = bytes.len().min(MAX_HANDLE_LEN - 1);
        handle_bytes[..len].copy_from_slice(&bytes[..len]);

        Self {
            meta_data_offset,
            mesh_shm_handle: handle_bytes,
        }
    }
}

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
    pub vert_count: u32,
    pub edge_count: u32,
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

            vert_count: total_verts,
            edge_count: total_edges,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct EngineCommand {
    pub should_cache: u16,
    pub op_id: u16,
    pub num_groups: u32,
    pub inline_data: [u8; MAX_INLINE_DATA],
}

impl EngineCommand {
    pub fn copy_payload_into_inline<T>(&mut self, payload: &[T])
    where
        T: Sized,
    {
        unsafe {
            let meta_ptr = payload.as_ptr() as *const u8;
            let meta_size = std::mem::size_of::<T>() * payload.len();
            std::ptr::copy_nonoverlapping(
                meta_ptr,
                self.inline_data.as_mut_ptr(),
                meta_size.min(MAX_INLINE_DATA),
            );
        }
    }

    pub fn get_meta_ptrs(&self) -> Vec<(SharedMemory, *mut AssetMeta)> {
        get_meta_ptrs(&self.inline_data, self.num_groups as usize)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct EngineResponse {
    pub status: u32, // 0 for OK, 1 for Error, etc.
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct MeshPublish {
    pub num_groups: u64,
    pub inline_data: [u8; MAX_INLINE_DATA],
}

impl MeshPublish {
    pub fn get_meta_ptrs(&self) -> Vec<( SharedMemory, *mut AssetMeta)> {
        get_meta_ptrs(&self.inline_data, self.num_groups as usize)
    }
}
// Common trait to parse `ShmOffset` entries from inline buffers.
fn get_meta_ptrs(inline_data: &[u8; MAX_INLINE_DATA], num_groups: usize) -> Vec<(SharedMemory, *mut AssetMeta)> {
    let asset_ptrs = unsafe {
        std::slice::from_raw_parts(
            inline_data.as_ptr() as *const ShmOffset,
            num_groups,
        )
    };

    let mut asset_meta_vec = Vec::with_capacity(num_groups);

    for ptr in asset_ptrs {
        let clean_handle_bytes = bytes_to_clean_str(&ptr.mesh_shm_handle);

        let shm_handle = String::from_utf8_lossy(clean_handle_bytes).to_string();

        let file_name = match FileName::new(shm_handle.as_bytes()) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("invalid shared memory name '{}': {:?}", shm_handle, e);
                continue;
            }
        };

        let shm = {
            SharedMemoryBuilder::new(&file_name)
                .open_existing(AccessMode::ReadWrite)
                .expect("Failed to open SHM")
        };

        let meta_ptr = unsafe {
            shm.base_address()
                .as_ptr()
                .add(ptr.meta_data_offset as usize) as *mut AssetMeta
        };

        asset_meta_vec.push((shm, meta_ptr));
    }
    asset_meta_vec
}

pub fn bytes_to_clean_str(bytes: &[u8]) -> &[u8] {
    // Look for the first null terminator, or use the whole slice if none found
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &bytes[..len]
}
