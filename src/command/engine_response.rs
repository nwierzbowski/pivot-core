use crate::{
    Buffer, BufferError, MAX_HANDLE_LEN,
    alloc::AllocResponseMeta,
    asset_ptr::AssetPtr,
    fields::Uuid,
    command::{EngineResponse, ResponseHeader},
};

impl EngineResponse {
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

    pub fn send_mesh() -> Self {
        Self::default()
    }

    pub fn read_send_mesh(&self) -> u16 {
        self.header.status
    }

    // --- 2. standardize_groups ----------------------------------------------

    pub fn standardize_groups() -> Self {
        Self::default()
    }

    pub fn read_standardize_groups(&self) -> u16 {
        self.header.status
    }

    // --- 3. standardize_synced_groups ---------------------------------------

    pub fn standardize_synced_groups() -> Self {
        Self::default()
    }

    pub fn read_standardize_synced_groups(&self) -> u16 {
        self.header.status
    }

    // --- 4. set_surface_types -----------------------------------------------

    pub fn set_surface_types() -> Self {
        Self::default()
    }

    pub fn read_set_surface_types(&self) -> u16 {
        self.header.status
    }

    // --- 5. drop_groups -----------------------------------------------------

    pub fn drop_groups() -> Self {
        Self::default()
    }

    pub fn read_drop_groups(&self) -> u16 {
        self.header.status
    }

    // --- 6. organize_objects ------------------------------------------------

    pub fn organize_objects() -> Self {
        Self::default()
    }

    pub fn read_organize_objects(&self) -> u16 {
        self.header.status
    }

    // --- 7. get_surface_types -----------------------------------------------

    pub fn get_surface_types() -> Self {
        Self::default()
    }

    pub fn read_get_surface_types(&self) -> u16 {
        self.header.status
    }

    // --- 8. stop_engine -----------------------------------------------------

    pub fn stop_engine() -> Self {
        Self::default()
    }

    pub fn read_stop_engine(&self) -> u16 {
        self.header.status
    }

    // --- 9. alloc_response --------------------------------------------------

    pub fn alloc_response(uuids: &[Uuid], ptrs: &[AssetPtr]) -> Self {
        if uuids.len() != ptrs.len() {
            panic!("alloc_response: uuids and ptrs must have same length");
        }
        let meta = AllocResponseMeta::new(uuids.len() as u64);
        let total = meta
            .offset_packed_ptrs
            .checked_add(
                ptrs.len()
                    .checked_mul(std::mem::size_of::<AssetPtr>())
                    .unwrap(),
            )
            .unwrap();
        if total > crate::MAX_INLINE_DATA {
            panic!("alloc_response: data exceeds buffer capacity");
        }
        let mut resp = Self::default();
        unsafe {
            let base = resp.inline_data.as_mut_ptr() as *mut u8;
            (base as *mut AllocResponseMeta).write(meta);
            std::ptr::copy_nonoverlapping(
                uuids.as_ptr() as *const u8,
                base.add(meta.offset_uuids),
                uuids.len() * std::mem::size_of::<Uuid>(),
            );
            std::ptr::copy_nonoverlapping(
                ptrs.as_ptr() as *const u8,
                base.add(meta.offset_packed_ptrs),
                ptrs.len() * std::mem::size_of::<AssetPtr>(),
            );
        }
        resp
    }

    pub fn read_alloc_response(&self) -> Result<(&[Uuid], &[AssetPtr]), BufferError> {
        let meta = unsafe {
            &*(self.inline_data.as_ptr() as *const AllocResponseMeta)
        };
        let num = meta.num_assets as usize;
        let ptr_end = meta
            .offset_packed_ptrs
            .checked_add(
                num.checked_mul(std::mem::size_of::<AssetPtr>())
                    .ok_or(BufferError::Corrupted)?,
            )
            .ok_or(BufferError::Corrupted)?;
        if ptr_end > crate::MAX_INLINE_DATA {
            return Err(BufferError::Corrupted);
        }
        let uuids = unsafe {
            std::slice::from_raw_parts(
                self.inline_data.as_ptr().add(meta.offset_uuids) as *const Uuid,
                num,
            )
        };
        let ptrs = unsafe {
            std::slice::from_raw_parts(
                self.inline_data.as_ptr().add(meta.offset_packed_ptrs) as *const AssetPtr,
                num,
            )
        };
        Ok((uuids, ptrs))
    }

    // --- 11. export_assets --------------------------------------------------

    pub fn export_assets(filenames: &[&str]) -> Self {
        let count = filenames.len() as u32;
        if 4 > crate::MAX_INLINE_DATA {
            panic!("export_assets: data exceeds buffer capacity");
        }
        let mut resp = Self::default();
        resp.inline_data.as_mut()[0..4].copy_from_slice(&count.to_le_bytes());
        let mut offset: usize = 4;
        for name in filenames {
            let name_len = name.len();
            if offset + name_len + 1 > crate::MAX_INLINE_DATA {
                panic!("export_assets: data exceeds buffer capacity");
            }
            unsafe {
                let base = resp.inline_data.as_mut_ptr().add(offset);
                std::ptr::copy_nonoverlapping(name.as_ptr(), base, name_len);
                *base.add(name_len) = 0;
            }
            offset += name_len + 1;
        }
        resp
    }

    pub fn read_export_assets(&self) -> Result<Vec<&str>, BufferError> {
        let data = self.inline_data.as_ref();
        if data.len() < 4 {
            return Err(BufferError::Corrupted);
        }
        let count = u32::from_le_bytes(data[0..4].try_into().map_err(|_| BufferError::Corrupted)?)
            as usize;
        let mut filenames = Vec::with_capacity(count);
        let mut offset = 4;
        for _ in 0..count {
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
            let name = std::str::from_utf8(&data[offset..offset + end])
                .map_err(|_| BufferError::InvalidUtf8)?;
            filenames.push(name);
            offset += end + 1;
        }
        Ok(filenames)
    }

    // --- 12. export_all -----------------------------------------------------

    pub fn export_all(filenames: &[&str]) -> Self {
        Self::export_assets(filenames)
    }

    pub fn read_export_all(&self) -> Result<Vec<&str>, BufferError> {
        self.read_export_assets()
    }

    // --- 13. import_assets --------------------------------------------------

    pub fn import_assets() -> Self {
        Self::default()
    }

    pub fn read_import_assets(&self) -> u16 {
        self.header.status
    }

    // --- 14. tbo_export_response --------------------------------------------
    // Layout:
    //   scene_count:     u64  @0
    //   asset_count:     u64  @8
    //   fragment_count:  u64  @16
    //   points_count:    u64  @24
    //   faces_count:     u64  @32
    //   scene_bytes:     u64  @40
    //   asset_bytes:     u64  @48
    //   fragment_bytes:  u64  @56
    //   points_bytes:    u64  @64
    //   faces_bytes:     u64  @72

    pub fn tbo_export_response(
        scene_count: u64,
        asset_count: u64,
        fragment_count: u64,
        points_count: u64,
        faces_count: u64,
        scene_bytes: u64,
        asset_bytes: u64,
        fragment_bytes: u64,
        points_bytes: u64,
        faces_bytes: u64,
    ) -> Self {
        let total = 80usize;
        if total > crate::MAX_INLINE_DATA {
            panic!("tbo_export_response: data exceeds buffer capacity");
        }
        let mut resp = Self::default();
        unsafe {
            let base = resp.inline_data.as_mut_ptr();
            *(base as *mut u64) = scene_count.to_le();
            *(base.add(8) as *mut u64) = asset_count.to_le();
            *(base.add(16) as *mut u64) = fragment_count.to_le();
            *(base.add(24) as *mut u64) = points_count.to_le();
            *(base.add(32) as *mut u64) = faces_count.to_le();
            *(base.add(40) as *mut u64) = scene_bytes.to_le();
            *(base.add(48) as *mut u64) = asset_bytes.to_le();
            *(base.add(56) as *mut u64) = fragment_bytes.to_le();
            *(base.add(64) as *mut u64) = points_bytes.to_le();
            *(base.add(72) as *mut u64) = faces_bytes.to_le();
        }
        resp
    }

    pub fn read_tbo_export_response(&self) -> Result<(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64), BufferError> {
        let data = self.inline_data.as_ref();
        let scene_count = u64::from_le_bytes(data[0..8].try_into().map_err(|_| BufferError::Corrupted)?);
        let asset_count = u64::from_le_bytes(data[8..16].try_into().map_err(|_| BufferError::Corrupted)?);
        let fragment_count = u64::from_le_bytes(data[16..24].try_into().map_err(|_| BufferError::Corrupted)?);
        let points_count = u64::from_le_bytes(data[24..32].try_into().map_err(|_| BufferError::Corrupted)?);
        let faces_count = u64::from_le_bytes(data[32..40].try_into().map_err(|_| BufferError::Corrupted)?);
        let scene_bytes = u64::from_le_bytes(data[40..48].try_into().map_err(|_| BufferError::Corrupted)?);
        let asset_bytes = u64::from_le_bytes(data[48..56].try_into().map_err(|_| BufferError::Corrupted)?);
        let fragment_bytes = u64::from_le_bytes(data[56..64].try_into().map_err(|_| BufferError::Corrupted)?);
        let points_bytes = u64::from_le_bytes(data[64..72].try_into().map_err(|_| BufferError::Corrupted)?);
        let faces_bytes = u64::from_le_bytes(data[72..80].try_into().map_err(|_| BufferError::Corrupted)?);
        Ok((scene_count, asset_count, fragment_count, points_count, faces_count, scene_bytes, asset_bytes, fragment_bytes, points_bytes, faces_bytes))
    }
}
