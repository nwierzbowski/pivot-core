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

    // --- 11. export_assets --------------------------------------------------

    pub fn export_assets(path: &str, uuids: &[Uuid]) -> Self {
        let path_len = path.len() + 1; // null terminator
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let aligned_uuid_start = Buffer::align_up(aligned_after_path, std::mem::size_of::<Uuid>());
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
            std::ptr::copy_nonoverlapping(
                uuids.as_ptr() as *const u8,
                base.add(aligned_uuid_start),
                uuid_size,
            );
        }
        cmd
    }

    pub fn read_export_assets(&self) -> Result<(&str, &[Uuid]), BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        let aligned_offset = Buffer::align_up(path_end + 1, 8);
        let uuid_start = Buffer::align_up(aligned_offset, std::mem::size_of::<Uuid>());
        let uuids = unsafe {
            std::slice::from_raw_parts(
                self.inline_data.as_ptr().add(uuid_start) as *const Uuid,
                self.num_headers as usize,
            )
        };
        Ok((path, uuids))
    }

    // --- 12. export_all -----------------------------------------------------

    pub fn export_all(path: &str) -> Self {
        let path_len = path.len() + 1;
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let total = aligned_after_path;
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
        }
        cmd
    }

    pub fn read_export_all(&self) -> Result<&str, BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        Ok(path)
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

    // --- 20. export_asset_tbo -----------------------------------------------

    pub fn export_asset_tbo(path: &str, uuids: &[Uuid]) -> Self {
        let path_len = path.len() + 1;
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let aligned_uuid_start = Buffer::align_up(aligned_after_path, std::mem::size_of::<Uuid>());
        let uuid_size = uuids.len() * std::mem::size_of::<Uuid>();
        let total = aligned_uuid_start + uuid_size;
        if total > crate::MAX_INLINE_DATA {
            panic!("export_asset_tbo: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_EXPORT_ASSET_TBO;
        cmd.num_headers = uuids.len() as u32;
        cmd.should_cache = 0;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(path.as_ptr(), base, path_len - 1);
            *base.add(path_len - 1) = 0;
            std::ptr::copy_nonoverlapping(
                uuids.as_ptr() as *const u8,
                base.add(aligned_uuid_start),
                uuid_size,
            );
        }
        cmd
    }

    pub fn read_export_asset_tbo(&self) -> Result<(&str, &[Uuid]), BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        let aligned_offset = Buffer::align_up(path_end + 1, 8);
        let uuid_start = Buffer::align_up(aligned_offset, std::mem::size_of::<Uuid>());
        let uuids = unsafe {
            std::slice::from_raw_parts(
                self.inline_data.as_ptr().add(uuid_start) as *const Uuid,
                self.num_headers as usize,
            )
        };
        Ok((path, uuids))
    }

    // --- 19. export_all_asset_tbo -------------------------------------------

    pub fn export_all_asset_tbo(path: &str, skip_normalization: bool) -> Self {
        let path_len = path.len() + 1;
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let total = aligned_after_path + 1; // path + skip_normalization
        if total > crate::MAX_INLINE_DATA {
            panic!("export_all_asset_tbo: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_EXPORT_ALL_ASSET_TBO;
        cmd.should_cache = 0;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(path.as_ptr(), base, path_len - 1);
            *base.add(path_len - 1) = 0;
            *base.add(aligned_after_path) = if skip_normalization { 1u8 } else { 0u8 };
        }
        cmd
    }

    pub fn read_export_all_asset_tbo(&self) -> Result<(&str, bool), BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        let aligned_offset = Buffer::align_up(path_end + 1, 8);
        let skip_normalization = self.inline_data.as_ref()[aligned_offset] != 0;
        Ok((path, skip_normalization))
    }

    // --- 16. export_all_asset_tbo_transforms --------------------------------

    pub fn export_all_asset_tbo_transforms(path: &str, scene_uuid: [u8; 32]) -> Self {
        let path_len = path.len() + 1;
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let scene_uuid_offset = aligned_after_path;
        let aligned_after_scene = Buffer::align_up(scene_uuid_offset + 32, 8);
        let total = aligned_after_scene; // path + scene_uuid
        if total > crate::MAX_INLINE_DATA {
            panic!("export_all_asset_tbo_transforms: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_EXPORT_ALL_ASSET_TBO_TRANSFORMS;
        cmd.should_cache = 0;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(path.as_ptr(), base, path_len - 1);
            *base.add(path_len - 1) = 0;
            std::ptr::copy_nonoverlapping(scene_uuid.as_ptr(), base.add(scene_uuid_offset), 32);
        }
        cmd
    }

    pub fn read_export_all_asset_tbo_transforms(&self) -> Result<(&str, [u8; 32]), BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        let aligned_offset = Buffer::align_up(path_end + 1, 8);
        let scene_uuid_offset = aligned_offset;
        let scene_uuid: [u8; 32] = self.inline_data.as_ref()[scene_uuid_offset..scene_uuid_offset + 32]
            .try_into()
            .map_err(|_| BufferError::Corrupted)?;
        Ok((path, scene_uuid))
    }

    // --- 15. drop_all_groups ------------------------------------------------

    pub fn drop_all_groups() -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_DROP_ALL_GROUPS;
        cmd
    }

    pub fn read_drop_all_groups(&self) {
        // No inline data
    }
    // --- 17. tbo_points_flush ------------------------------------------------

    pub fn tbo_points_flush(path: &str, channel_mask: u32, target_point_count: u32) -> Self {
        let path_len = path.len() + 1;
        let aligned_after_path = Buffer::align_up(path_len, 8);
        let total = aligned_after_path + 4 + 4; // path + channel_mask + target_point_count
        if total > crate::MAX_INLINE_DATA {
            panic!("tbo_points_flush: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_TBO_POINTS_FLUSH;
        cmd.should_cache = 0;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(path.as_ptr(), base, path_len - 1);
            *base.add(path_len - 1) = 0;
            *(base.add(aligned_after_path) as *mut u32) = channel_mask;
            *(base.add(aligned_after_path + 4) as *mut u32) = target_point_count;
        }
        cmd
    }

    pub fn read_tbo_points_flush(&self) -> Result<(&str, u32, u32), BufferError> {
        let path_end = self
            .inline_data
            .as_ref()
            .iter()
            .position(|&b| b == 0)
            .ok_or(BufferError::Corrupted)?;
        let path = std::str::from_utf8(&self.inline_data.as_ref()[..path_end])
            .map_err(|_| BufferError::InvalidUtf8)?;
        let aligned_offset = Buffer::align_up(path_end + 1, 8);
        let channel_mask = u32::from_le_bytes(
            self.inline_data.as_ref()[aligned_offset..aligned_offset + 4]
                .try_into()
                .map_err(|_| BufferError::Corrupted)?,
        );
        let target_point_count = u32::from_le_bytes(
            self.inline_data.as_ref()[aligned_offset + 4..aligned_offset + 8]
                .try_into()
                .map_err(|_| BufferError::Corrupted)?,
        );
        Ok((path, channel_mask, target_point_count))
    }

    // --- 20. group_all_objects ----------------------------------------------

    pub fn group_all_objects() -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_GROUP_ALL_OBJECTS;
        cmd
    }

    pub fn read_group_all_objects(&self) {
        // No inline data
    }

    // --- 23. embed_all_assets -----------------------------------------------

    pub fn embed_all_assets(should_cache: u16) -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_EMBED_ALL_ASSETS;
        cmd.should_cache = should_cache;
        cmd
    }

    pub fn read_embed_all_assets(&self) {
        // No inline data
    }

  }
