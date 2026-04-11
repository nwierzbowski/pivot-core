use crate::{
    Buffer, BufferError, MAX_HANDLE_LEN,
    asset_ptr::AssetPtr,
    command::{MeshPublish, ResponseHeader},
};

impl MeshPublish {
    fn default() -> Self {
        Self {
            header: ResponseHeader {
                status: 0,
                total_slabs: 0,
                num_items: 0,
                root_slab_handle: [0u8; MAX_HANDLE_LEN],
            },
            inline_data: Buffer::new(),
        }
    }

    // --- 1. send_mesh -------------------------------------------------------

    pub fn send_mesh(ptrs: &[AssetPtr]) -> Self {
        let mut mesh = Self::default();
        mesh.header.num_items = ptrs.len() as u32;
        let _ = mesh.inline_data.write_asset_ptrs(ptrs);
        mesh
    }

    pub fn read_send_mesh(&self) -> Result<&[AssetPtr], BufferError> {
        self.inline_data
            .read_asset_ptrs(self.header.num_items as usize)
    }

    // --- 2. standardize_groups ----------------------------------------------

    pub fn standardize_groups() -> Self {
        Self::default()
    }

    pub fn read_standardize_groups(&self) -> u16 {
        self.header.status
    }

    // --- 3. organize_objects ------------------------------------------------

    pub fn organize_objects() -> Self {
        Self::default()
    }

    pub fn read_organize_objects(&self) -> u16 {
        self.header.status
    }

    // --- 4. extract_geometric_features --------------------------------------

    pub fn extract_geometric_features() -> Self {
        Self::default()
    }

    pub fn read_extract_geometric_features(&self) -> u16 {
        self.header.status
    }
}
