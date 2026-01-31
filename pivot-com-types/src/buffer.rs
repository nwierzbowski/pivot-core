use bytemuck::{Pod, Zeroable, cast_slice, from_bytes};
use iceoryx2::prelude::*;
use iceoryx2_bb_posix::{
    file::AccessMode,
    shared_memory::{SharedMemory, SharedMemoryBuilder},
};

use crate::{
    alloc::{AllocRequestMeta, AllocResponseMeta}, asset_meta::AssetMeta, asset_ptr::AssetPtr,
    asset_surface::GroupSurface, fields::Uuid,
};

pub const MAX_INLINE_DATA: usize = 65536; // 64 KB (L1 Cache Friendly)

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, ZeroCopySend)]
pub struct Buffer {
    data: [u8; MAX_INLINE_DATA],
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            data: [0u8; MAX_INLINE_DATA],
        }
    }

    pub fn copy_payload<T>(&mut self, payload: &[T], offset: usize)
    where
        T: Sized,
    {
        unsafe {
            let ptr = payload.as_ptr() as *const u8;
            let t_size = std::mem::size_of::<T>() * payload.len();
            std::ptr::copy_nonoverlapping(
                ptr,
                self.data.as_mut_ptr().add(offset),
                t_size.min(MAX_INLINE_DATA - offset),
            );
        }
    }

    pub fn to_alloc_request(&self) {
        let req = from_bytes::<AllocRequestMeta>(&self.data);
    }

    pub fn to_alloc_response(&self) -> (Vec<Uuid>, Vec<AssetPtr>) {
        let resp = from_bytes::<AllocResponseMeta>(&self.data);
        let ptrs = cast_slice::<u8, AssetPtr>(&self.data[resp.offset_packed_ptrs..resp.offset_packed_ptrs + (resp.num_assets as usize * size_of::<AssetPtr>())]);
        let uuids = cast_slice::<u8, Uuid>(&self.data[resp.offset_uuids..resp.offset_uuids + (resp.num_assets as usize * size_of::<Uuid>())]);
        (uuids.to_vec(), ptrs.to_vec())
    }

    pub fn to_asset_meta_ptr(&self, num_groups: usize) -> Vec<(SharedMemory, *mut AssetMeta)> {
        let asset_ptrs = unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const AssetPtr, num_groups)
        };
        let mut asset_meta_vec = Vec::with_capacity(num_groups);

        // for ptr in asset_ptrs {
        //     let clean_handle_bytes = bytes_to_clean_str(&ptr.mesh_shm_handle);

        //     let shm_handle = String::from_utf8_lossy(clean_handle_bytes).to_string();

        //     let file_name = match FileName::new(shm_handle.as_bytes()) {
        //         Ok(f) => f,
        //         Err(e) => {
        //             eprintln!("invalid shared memory name '{}': {:?}", shm_handle, e);
        //             continue;
        //         }
        //     };

        //     let shm = {
        //         SharedMemoryBuilder::new(&file_name)
        //             .open_existing(AccessMode::ReadWrite)
        //             .expect("Failed to open SHM")
        //     };

        //     let meta_ptr = unsafe {
        //         shm.base_address()
        //             .as_ptr()
        //             .add(ptr.meta_data_offset as usize) as *mut AssetMeta
        //     };

        //     asset_meta_vec.push((shm, meta_ptr));
        // }
        asset_meta_vec
    }

    pub fn to_group_surfaces(&self, num_groups: usize) -> &[GroupSurface] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const GroupSurface,
                num_groups as usize,
            )
        }
    }

    pub fn to_group_names(&self, num_groups: usize) -> Vec<String> {
        let group_surfaces = self.to_group_surfaces(num_groups);
        group_surfaces
            .iter()
            .map(|group| {
                let clean_name_bytes = bytes_to_clean_str(&group.group_name);
                String::from_utf8_lossy(clean_name_bytes).to_string()
            })
            .collect()
    }
}

pub fn bytes_to_clean_str(bytes: &[u8]) -> &[u8] {
    // Look for the first null terminator, or use the whole slice if none found
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &bytes[..len]
}
