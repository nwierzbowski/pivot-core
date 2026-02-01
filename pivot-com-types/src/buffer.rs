use bytemuck::{Pod, Zeroable};
use iceoryx2::prelude::*;

use crate::{
    alloc::{AllocRequestMeta, AllocResponseMeta}, asset_ptr::AssetPtr, asset_surface::GroupSurface, fields::Uuid
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

    pub fn to_alloc_request(&self) -> (&[Uuid], &[usize]) {
        let req = unsafe { &*(self.data.as_ptr() as *const AllocRequestMeta) };
        let base_ptr = self.data.as_ptr();
        let num = req.num_assets as usize;
        unsafe {
            let uuids =
                std::slice::from_raw_parts(base_ptr.add(req.offset_uuids) as *const Uuid, num);
            let sizes =
                std::slice::from_raw_parts(base_ptr.add(req.offset_sizes) as *const usize, num);
            (uuids, sizes)
        }
    }

    pub fn to_alloc_response(&self) -> (&[Uuid], &[AssetPtr]) {
        let resp = unsafe { &*(self.data.as_ptr() as *const AllocResponseMeta) };
        let base_ptr = self.data.as_ptr();
        let num = resp.num_assets as usize;
        let uuids = unsafe {
            std::slice::from_raw_parts(base_ptr.add(resp.offset_uuids) as *const Uuid, num)
        };
        let ptrs = unsafe {
            std::slice::from_raw_parts(
                base_ptr.add(resp.offset_packed_ptrs) as *const AssetPtr,
                num,
            )
        };
        (uuids, ptrs)
    }

    pub fn to_asset_meta_ptr(&self, num_groups: usize) -> &[AssetPtr] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const AssetPtr, num_groups) }
    }

    pub fn to_group_surfaces(&self, num_groups: usize) -> &[GroupSurface] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const GroupSurface,
                num_groups as usize,
            )
        }
    }

    pub fn to_uuids(&self, num_groups: usize) ->&[Uuid] {
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const Uuid, num_groups as usize)
        }
    }
}

pub fn bytes_to_clean_str(bytes: &[u8]) -> &[u8] {
    // Look for the first null terminator, or use the whole slice if none found
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &bytes[..len]
}
