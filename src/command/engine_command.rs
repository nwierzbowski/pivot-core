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

    // --- 15. drop_all_groups ------------------------------------------------

    pub fn drop_all_groups() -> Self {
        let mut cmd = Self::default();
        cmd.op_id = OP_DROP_ALL_GROUPS;
        cmd
    }

    pub fn read_drop_all_groups(&self) {
        // No inline data
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

    // --- 23. tbo_export -------------------------------------------------------
    // Per format: data_ptr (engine writes f32 data here →),
    //             offset_ptr (engine writes u64 offsets here ←),
    //             remaining (gap between data_ptr and offset_ptr)
    // Layout:
    //   slab_scene_name:     [u8; 64]  @0
    //   slab_asset_name:     [u8; 64]  @64
    //   slab_fragment_name:  [u8; 64]  @128
    //   slab_points_name:    [u8; 64]  @192
    //   slab_faces_name:     [u8; 64]  @256
    //   scene_data_ptr:      u64       @320
    //   scene_offset_ptr:    u64       @328
    //   scene_remaining:     u64       @336
    //   asset_data_ptr:      u64       @344
    //   asset_offset_ptr:    u64       @352
    //   asset_remaining:     u64       @360
    //   frag_data_ptr:       u64       @368
    //   frag_offset_ptr:     u64       @376
    //   frag_remaining:      u64       @384
    //   points_data_ptr:     u64       @392
    //   points_offset_ptr:   u64       @400
    //   points_remaining:    u64       @408
    //   faces_data_ptr:      u64       @416
    //   faces_offset_ptr:    u64       @424
    //   faces_remaining:     u64       @432
    //   flags:               10 × u8   @440
    //   target_point_count:  u32       @450 (aligned @456)

    pub fn tbo_export(
        slab_scene_name: &[u8; 64],
        slab_asset_name: &[u8; 64],
        slab_fragment_name: &[u8; 64],
        slab_points_name: &[u8; 64],
        slab_faces_name: &[u8; 64],
        scene_data_ptr: u64,
        scene_offset_ptr: u64,
        scene_remaining: u64,
        asset_data_ptr: u64,
        asset_offset_ptr: u64,
        asset_remaining: u64,
        frag_data_ptr: u64,
        frag_offset_ptr: u64,
        frag_remaining: u64,
        points_data_ptr: u64,
        points_offset_ptr: u64,
        points_remaining: u64,
        faces_data_ptr: u64,
        faces_offset_ptr: u64,
        faces_remaining: u64,
        scene_transform: bool,
        scene_similarity: bool,
        asset_embedding: bool,
        asset_transform: bool,
        fragment_xyz: bool,
        normal_variance: bool,
        surface_variation: bool,
        combined: bool,
        points_original: bool,
        faces: bool,
        target_point_count: u32,
    ) -> Self {
        let total = Buffer::align_up(450 + 4, 8);
        if total > crate::MAX_INLINE_DATA {
            panic!("tbo_export: data exceeds buffer capacity");
        }
        let mut cmd = Self::default();
        cmd.op_id = OP_TBO_EXPORT;
        cmd.should_cache = 0;
        unsafe {
            let base = cmd.inline_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(slab_scene_name.as_ptr(), base, 64);
            std::ptr::copy_nonoverlapping(slab_asset_name.as_ptr(), base.add(64), 64);
            std::ptr::copy_nonoverlapping(slab_fragment_name.as_ptr(), base.add(128), 64);
            std::ptr::copy_nonoverlapping(slab_points_name.as_ptr(), base.add(192), 64);
            std::ptr::copy_nonoverlapping(slab_faces_name.as_ptr(), base.add(256), 64);
            *(base.add(320) as *mut u64) = scene_data_ptr.to_le();
            *(base.add(328) as *mut u64) = scene_offset_ptr.to_le();
            *(base.add(336) as *mut u64) = scene_remaining.to_le();
            *(base.add(344) as *mut u64) = asset_data_ptr.to_le();
            *(base.add(352) as *mut u64) = asset_offset_ptr.to_le();
            *(base.add(360) as *mut u64) = asset_remaining.to_le();
            *(base.add(368) as *mut u64) = frag_data_ptr.to_le();
            *(base.add(376) as *mut u64) = frag_offset_ptr.to_le();
            *(base.add(384) as *mut u64) = frag_remaining.to_le();
            *(base.add(392) as *mut u64) = points_data_ptr.to_le();
            *(base.add(400) as *mut u64) = points_offset_ptr.to_le();
            *(base.add(408) as *mut u64) = points_remaining.to_le();
            *(base.add(416) as *mut u64) = faces_data_ptr.to_le();
            *(base.add(424) as *mut u64) = faces_offset_ptr.to_le();
            *(base.add(432) as *mut u64) = faces_remaining.to_le();
            let flags = base.add(440);
            *flags.add(0) = if scene_transform { 1 } else { 0 };
            *flags.add(1) = if scene_similarity { 1 } else { 0 };
            *flags.add(2) = if asset_embedding { 1 } else { 0 };
            *flags.add(3) = if asset_transform { 1 } else { 0 };
            *flags.add(4) = if fragment_xyz { 1 } else { 0 };
            *flags.add(5) = if normal_variance { 1 } else { 0 };
            *flags.add(6) = if surface_variation { 1 } else { 0 };
            *flags.add(7) = if combined { 1 } else { 0 };
            *flags.add(8) = if points_original { 1 } else { 0 };
            *flags.add(9) = if faces { 1 } else { 0 };
            *(base.add(452) as *mut u32) = target_point_count.to_le();
        }
        cmd
    }

    pub fn read_tbo_export(&self) -> Result<(
        [u8; 64], [u8; 64], [u8; 64], [u8; 64], [u8; 64],
        u64, u64, u64,
        u64, u64, u64,
        u64, u64, u64,
        u64, u64, u64,
        u64, u64, u64,
        bool, bool,
        bool, bool,
        bool, bool, bool, bool,
        bool, bool,
        u32,
    ), BufferError> {
        let data = self.inline_data.as_ref();
        let slab_scene_name: [u8; 64] = data[0..64].try_into().map_err(|_| BufferError::Corrupted)?;
        let slab_asset_name: [u8; 64] = data[64..128].try_into().map_err(|_| BufferError::Corrupted)?;
        let slab_fragment_name: [u8; 64] = data[128..192].try_into().map_err(|_| BufferError::Corrupted)?;
        let slab_points_name: [u8; 64] = data[192..256].try_into().map_err(|_| BufferError::Corrupted)?;
        let slab_faces_name: [u8; 64] = data[256..320].try_into().map_err(|_| BufferError::Corrupted)?;
        let scene_data_ptr = u64::from_le_bytes(data[320..328].try_into().map_err(|_| BufferError::Corrupted)?);
        let scene_offset_ptr = u64::from_le_bytes(data[328..336].try_into().map_err(|_| BufferError::Corrupted)?);
        let scene_remaining = u64::from_le_bytes(data[336..344].try_into().map_err(|_| BufferError::Corrupted)?);
        let asset_data_ptr = u64::from_le_bytes(data[344..352].try_into().map_err(|_| BufferError::Corrupted)?);
        let asset_offset_ptr = u64::from_le_bytes(data[352..360].try_into().map_err(|_| BufferError::Corrupted)?);
        let asset_remaining = u64::from_le_bytes(data[360..368].try_into().map_err(|_| BufferError::Corrupted)?);
        let frag_data_ptr = u64::from_le_bytes(data[368..376].try_into().map_err(|_| BufferError::Corrupted)?);
        let frag_offset_ptr = u64::from_le_bytes(data[376..384].try_into().map_err(|_| BufferError::Corrupted)?);
        let frag_remaining = u64::from_le_bytes(data[384..392].try_into().map_err(|_| BufferError::Corrupted)?);
        let points_data_ptr = u64::from_le_bytes(data[392..400].try_into().map_err(|_| BufferError::Corrupted)?);
        let points_offset_ptr = u64::from_le_bytes(data[400..408].try_into().map_err(|_| BufferError::Corrupted)?);
        let points_remaining = u64::from_le_bytes(data[408..416].try_into().map_err(|_| BufferError::Corrupted)?);
        let faces_data_ptr = u64::from_le_bytes(data[416..424].try_into().map_err(|_| BufferError::Corrupted)?);
        let faces_offset_ptr = u64::from_le_bytes(data[424..432].try_into().map_err(|_| BufferError::Corrupted)?);
        let faces_remaining = u64::from_le_bytes(data[432..440].try_into().map_err(|_| BufferError::Corrupted)?);
        let scene_transform = data[440] != 0;
        let scene_similarity = data[441] != 0;
        let asset_embedding = data[442] != 0;
        let asset_transform = data[443] != 0;
        let fragment_xyz = data[444] != 0;
        let normal_variance = data[445] != 0;
        let surface_variation = data[446] != 0;
        let combined = data[447] != 0;
        let points_original = data[448] != 0;
        let faces = data[449] != 0;
        let target_point_count = u32::from_le_bytes(
            data[452..456].try_into().map_err(|_| BufferError::Corrupted)?,
        );
        Ok((
            slab_scene_name, slab_asset_name, slab_fragment_name, slab_points_name, slab_faces_name,
            scene_data_ptr, scene_offset_ptr, scene_remaining,
            asset_data_ptr, asset_offset_ptr, asset_remaining,
            frag_data_ptr, frag_offset_ptr, frag_remaining,
            points_data_ptr, points_offset_ptr, points_remaining,
            faces_data_ptr, faces_offset_ptr, faces_remaining,
            scene_transform, scene_similarity,
            asset_embedding, asset_transform,
            fragment_xyz, normal_variance, surface_variation, combined,
            points_original, faces,
            target_point_count,
        ))
    }

}
