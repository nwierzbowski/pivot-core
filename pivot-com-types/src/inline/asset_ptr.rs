use crate::constants::MAX_HANDLE_LEN;

#[repr(C)]
pub struct AssetPtr {
    pub meta_data_offset: u64,
    pub mesh_shm_handle: [u8; MAX_HANDLE_LEN], // The single SHM containing ALL data for this group
}

impl AssetPtr {
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