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

    /// Export request: [null-terminated path][4-byte target_bytes][AssetPtrs...]
    pub fn to_export_params(&self) -> (&str, u32) {
        let data = &self.data;
        let path_end = data.iter().position(|&b| b == 0).unwrap_or(MAX_INLINE_DATA);
        let path = unsafe { std::str::from_utf8_unchecked(&data[..path_end]) };
        let target_bytes = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr().add(path_end + 1) as *const u32,
                1
            )[0]
        };
        (path, target_bytes)
    }

    /// Extracts AssetPtrs starting after path+target_bytes
    pub fn to_export_ptrs(&self, num_ptrs: usize) -> &[AssetPtr] {
        let data = &self.data;
        let path_end = data.iter().position(|&b| b == 0).unwrap_or(MAX_INLINE_DATA);
        let ptr_start = path_end + 1 + 4; // null + target_bytes
        unsafe {
            std::slice::from_raw_parts(
                data.as_ptr().add(ptr_start) as *const AssetPtr,
                num_ptrs
            )
        }
    }

    /// Import request: [null-terminated path][null-terminated path]...
    pub fn to_import_paths(&self, num_paths: usize) -> Vec<&str> {
        let mut paths = Vec::with_capacity(num_paths);
        let mut offset = 0;
        let data = &self.data;
        
        for _ in 0..num_paths {
            if offset >= MAX_INLINE_DATA { break; }
            let path_end = data[offset..].iter().position(|&b| b == 0)
                .unwrap_or(MAX_INLINE_DATA - offset);
            if path_end == 0 { break; }
            let path = unsafe { std::str::from_utf8_unchecked(&data[offset..offset+path_end]) };
            paths.push(path);
            offset += path_end + 1;
        }
        paths
    }

    /// Export response: [4-byte filename_count][null-terminated filename]...
    pub fn to_export_response(&self) -> Vec<&str> {
        let data = &self.data;
        let mut count_buf = [0u8; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), count_buf.as_mut_ptr(), 4);
        }
        let count = u32::from_le_bytes(count_buf);
        let mut filenames = Vec::with_capacity(count as usize);
        let mut offset = 4;
        
        for _ in 0..count {
            if offset >= MAX_INLINE_DATA { break; }
            let end = data[offset..].iter().position(|&b| b == 0)
                .unwrap_or(MAX_INLINE_DATA - offset);
            if end == 0 { break; }
            let name = unsafe { std::str::from_utf8_unchecked(&data[offset..offset+end]) };
            filenames.push(name);
            offset += end + 1;
        }
        filenames
    }
}

// pub fn bytes_to_clean_str(bytes: &[u8]) -> &[u8] {
//     // Look for the first null terminator, or use the whole slice if none found
//     let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
//     &bytes[..len]
// }
