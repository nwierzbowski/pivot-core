use crate::{
    Buffer, BufferError,
    alloc::AllocRequestMeta,
    asset_ptr::AssetPtr,
    asset_surface::GroupSurface,
    fields::Uuid,
    command::{EngineCommand, *},
};

impl EngineCommand {
    fn default() -> Self {
        Self {
            should_cache: 0,
            op_id: 0,
            num_headers: 0,
            inline_data: Buffer::new(),
        }
    }

    // --- 1. send_mesh -------------------------------------------------------

    pub fn send_mesh(ptrs: &[AssetPtr]) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_SEND_MESH;
        cmd.num_headers = ptrs.len() as u32;
        cmd.should_cache = 0;
        let _ = cmd.inline_data.write_asset_ptrs(ptrs);
        cmd
    }

    pub fn read_send_mesh(&self) -> Result<&[AssetPtr], BufferError> {
        self.inline_data.read_asset_ptrs(self.num_headers as usize)
    }

    // --- 2. standardize_groups ----------------------------------------------

    pub fn standardize_groups(uuids: &[Uuid]) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_STANDARDIZE_GROUPS;
        cmd.num_headers = uuids.len() as u32;
        cmd.should_cache = 0;
        let _ = cmd.inline_data.write_uuids(uuids);
        cmd
    }

    pub fn read_standardize_groups(&self) -> Result<&[Uuid], BufferError> {
        self.inline_data.read_uuids(self.num_headers as usize)
    }

    // --- 3. standardize_synced_groups ---------------------------------------

    pub fn standardize_synced_groups(surfaces: &[GroupSurface], should_cache: u16) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_STANDARDIZE_SYNCED_GROUPS;
        cmd.num_headers = surfaces.len() as u32;
        cmd.should_cache = should_cache;
        let _ = cmd.inline_data.write_group_surfaces(surfaces);
        cmd
    }

    pub fn read_standardize_synced_groups(&self) -> Result<&[GroupSurface], BufferError> {
        self.inline_data.read_group_surfaces(self.num_headers as usize)
    }

    // --- 4. set_surface_types -----------------------------------------------

    pub fn set_surface_types(surfaces: &[GroupSurface], should_cache: u16) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_SET_SURFACE_TYPES;
        cmd.num_headers = surfaces.len() as u32;
        cmd.should_cache = should_cache;
        let _ = cmd.inline_data.write_group_surfaces(surfaces);
        cmd
    }

    pub fn read_set_surface_types(&self) -> Result<&[GroupSurface], BufferError> {
        self.inline_data.read_group_surfaces(self.num_headers as usize)
    }

    // --- 5. drop_groups -----------------------------------------------------

    pub fn drop_groups(uuids: &[Uuid], should_cache: u16) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_DROP_GROUPS;
        cmd.num_headers = uuids.len() as u32;
        cmd.should_cache = should_cache;
        let _ = cmd.inline_data.write_uuids(uuids);
        cmd
    }

    pub fn read_drop_groups(&self) -> Result<&[Uuid], BufferError> {
        self.inline_data.read_uuids(self.num_headers as usize)
    }

    // --- 6. organize_objects ------------------------------------------------

    pub fn organize_objects(should_cache: u16) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_ORGANIZE_OBJECTS;
        cmd.should_cache = should_cache;
        cmd
    }

    pub fn read_organize_objects(&self) {
        // No inline data
    }

    // --- 7. get_surface_types -----------------------------------------------

    pub fn get_surface_types(should_cache: u16) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_GET_SURFACE_TYPES;
        cmd.should_cache = should_cache;
        cmd
    }

    pub fn read_get_surface_types(&self) {
        // No inline data
    }

    // --- 8. stop_engine -----------------------------------------------------

    pub fn stop_engine() -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_STOP_ENGINE;
        cmd
    }

    pub fn read_stop_engine(&self) {
        // No inline data
    }

    // --- 9. alloc_request ---------------------------------------------------

    pub fn alloc_request(uuids: &[Uuid], sizes: &[usize]) -> Self {
        if uuids.len() != sizes.len() {
            panic!("alloc_request: uuids and sizes must have same length");
        }
        let meta = AllocRequestMeta::new(uuids.len() as u64);
        let total = meta
            .offset_sizes
            .checked_add(sizes.len().checked_mul(std::mem::size_of::<usize>()).unwrap())
            .unwrap();
        if total > crate::MAX_INLINE_DATA {
            panic!("alloc_request: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_ALLOC_MEM;
        cmd.num_headers = 1;
        cmd.should_cache = 1;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr() as *mut u8;
            (base as *mut AllocRequestMeta).write(meta);
            std::ptr::copy_nonoverlapping(
                uuids.as_ptr() as *const u8,
                base.add(meta.offset_uuids),
                uuids.len() * std::mem::size_of::<Uuid>(),
            );
            std::ptr::copy_nonoverlapping(
                sizes.as_ptr() as *const u8,
                base.add(meta.offset_sizes),
                sizes.len() * std::mem::size_of::<usize>(),
            );
        }
        cmd
    }

    pub fn read_alloc_request(&self) -> Result<(&[Uuid], &[usize]), BufferError> {
        let meta = unsafe {
            &*(self.inline_data.as_ptr() as *const AllocRequestMeta)
        };
        let num = meta.num_assets as usize;
        let size_end = meta
            .offset_sizes
            .checked_add(num.checked_mul(std::mem::size_of::<usize>()).ok_or(BufferError::Corrupted)?)
            .ok_or(BufferError::Corrupted)?;
        if size_end > crate::MAX_INLINE_DATA {
            return Err(BufferError::Corrupted);
        }
        let uuids = unsafe {
            std::slice::from_raw_parts(
                self.inline_data.as_ptr().add(meta.offset_uuids) as *const Uuid,
                num,
            )
        };
        let sizes = unsafe {
            std::slice::from_raw_parts(
                self.inline_data.as_ptr().add(meta.offset_sizes) as *const usize,
                num,
            )
        };
        Ok((uuids, sizes))
    }

    // --- 10. extract_geometric_features -------------------------------------

    pub fn extract_geometric_features(uuids: &[Uuid], should_cache: u16) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_EXTRACT_GEOMETRIC_FEATURES;
        cmd.num_headers = uuids.len() as u32;
        cmd.should_cache = should_cache;
        let _ = cmd.inline_data.write_uuids(uuids);
        cmd
    }

    pub fn read_extract_geometric_features(&self) -> Result<&[Uuid], BufferError> {
        self.inline_data.read_uuids(self.num_headers as usize)
    }

    // --- 11. export_assets --------------------------------------------------

    pub fn export_assets(path: &str, target_bytes: u64, uuids: &[Uuid]) -> Self {
        let path_len = path.len() + 1; // null terminator
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let after_target = aligned_after_path + 8;
        let aligned_uuid_start = Buffer::align_up(after_target, std::mem::size_of::<Uuid>());
        let uuid_size = uuids.len() * std::mem::size_of::<Uuid>();
        let total = aligned_uuid_start + uuid_size;
        if total > crate::MAX_INLINE_DATA {
            panic!("export_assets: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_EXPORT_ASSETS;
        cmd.num_headers = uuids.len() as u32;
        cmd.should_cache = 0;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(path.as_ptr(), base, path_len - 1);
            *base.add(path_len - 1) = 0;
            *(base.add(aligned_after_path) as *mut u64) = target_bytes;
            std::ptr::copy_nonoverlapping(
                uuids.as_ptr() as *const u8,
                base.add(aligned_uuid_start),
                uuid_size,
            );
        }
        cmd
    }

    pub fn read_export_assets(&self) -> Result<(&str, u64, &[Uuid]), BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        let aligned_offset = Buffer::align_up(path_end + 1, 8);
        if aligned_offset + 8 > crate::MAX_INLINE_DATA {
            return Err(BufferError::Corrupted);
        }
        let target_bytes = u64::from_le_bytes(
            self.inline_data.as_ref()[aligned_offset..aligned_offset + 8]
                .try_into()
                .map_err(|_| BufferError::Corrupted)?,
        );
        let uuid_start = Buffer::align_up(aligned_offset + 8, std::mem::size_of::<Uuid>());
        let uuids = unsafe {
            std::slice::from_raw_parts(
                self.inline_data.as_ptr().add(uuid_start) as *const Uuid,
                self.num_headers as usize,
            )
        };
        Ok((path, target_bytes, uuids))
    }

    // --- 12. export_all -----------------------------------------------------

    pub fn export_all(path: &str, target_bytes: u64) -> Self {
        let path_len = path.len() + 1;
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let total = aligned_after_path + 8;
        if total > crate::MAX_INLINE_DATA {
            panic!("export_all: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_EXPORT_ALL;
        cmd.should_cache = 0;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(path.as_ptr(), base, path_len - 1);
            *base.add(path_len - 1) = 0;
            *(base.add(aligned_after_path) as *mut u64) = target_bytes;
        }
        cmd
    }

    pub fn read_export_all(&self) -> Result<(&str, u64), BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        let aligned_offset = Buffer::align_up(path_end + 1, 8);
        if aligned_offset + 8 > crate::MAX_INLINE_DATA {
            return Err(BufferError::Corrupted);
        }
        let target_bytes = u64::from_le_bytes(
            self.inline_data.as_ref()[aligned_offset..aligned_offset + 8]
                .try_into()
                .map_err(|_| BufferError::Corrupted)?,
        );
        Ok((path, target_bytes))
    }

    // --- 13. import_assets --------------------------------------------------

    pub fn import_assets(paths: &[&str]) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_IMPORT_ASSETS;
        let mut offset: usize = 0;
        for path in paths {
            let path_len = path.len();
            if offset + path_len + 1 > crate::MAX_INLINE_DATA {
                panic!("import_assets: data exceeds buffer capacity");
            }
            unsafe {
                let base = cmd.inline_data.as_mut_ptr().add(offset);
                std::ptr::copy_nonoverlapping(path.as_ptr(), base, path_len);
                *base.add(path_len) = 0;
            }
            offset += path_len + 1;
        }
        cmd.num_headers = paths.len() as u32;
        cmd
    }

    pub fn read_import_assets(&self) -> Result<Vec<&str>, BufferError> {
        let mut result = Vec::with_capacity(self.num_headers as usize);
        let mut offset = 0;
        let data = self.inline_data.as_ref();
        for _ in 0..self.num_headers {
            if offset >= data.len() {
                break;
            }
            let end = data[offset..]
                .iter()
                .position(|&b| b == 0)
                .ok_or(BufferError::Corrupted)?;
            if end == 0 {
                break;
            }
            let path = std::str::from_utf8(&data[offset..offset + end])
                .map_err(|_| BufferError::InvalidUtf8)?;
            result.push(path);
            offset += end + 1;
        }
        Ok(result)
    }
}
