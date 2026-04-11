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

    /// Align offset up to the nearest multiple of `align`
    pub fn align_up(offset: usize, align: usize) -> usize {
        (offset + align - 1) & !(align - 1)
    }

    /// Export request: [null-terminated path][4-byte target_bytes][AssetPtrs...]
    /// Assumes target_bytes is placed at an aligned offset after the null terminator
    pub fn to_export_params(&self) -> (&str, u32) {
        let data = &self.data;
        let path_end = data.iter().position(|&b| b == 0).unwrap_or(MAX_INLINE_DATA);
        let path = unsafe { std::str::from_utf8_unchecked(&data[..path_end]) };
        // Align to u32 boundary after null terminator
        let aligned_offset = Self::align_up(path_end + 1, std::mem::align_of::<u32>());
        let target_bytes = unsafe {
            *(data.as_ptr().add(aligned_offset) as *const u32)
        };
        (path, target_bytes)
    }

    /// Extracts AssetPtrs starting after path+target_bytes
    /// Assumes AssetPtrs are placed at an aligned offset after target_bytes
    pub fn to_export_ptrs(&self, num_ptrs: usize) -> &[AssetPtr] {
        let data = &self.data;
        let path_end = data.iter().position(|&b| b == 0).unwrap_or(MAX_INLINE_DATA);
        // Align target_bytes, then add 4 bytes for the value, then align again for AssetPtrs
        let target_aligned = Self::align_up(path_end + 1, std::mem::align_of::<u32>());
        let ptr_start = Self::align_up(target_aligned + 4, std::mem::align_of::<AssetPtr>());
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_export_params() {
        let mut buf = Buffer::new();
        let path = "/tmp/test_export";
        let target_bytes: u32 = 1024 * 1024; // 1MB

        // Write path (null-terminated)
        for (i, &b) in path.as_bytes().iter().enumerate() {
            buf.data[i] = b;
        }
        buf.data[path.len()] = 0;
        
        // Write target_bytes at aligned offset after null terminator
        let aligned_offset = Buffer::align_up(path.len() + 1, std::mem::align_of::<u32>());
        unsafe {
            *(buf.data.as_mut_ptr().add(aligned_offset) as *mut u32) = target_bytes;
        }

        let (parsed_path, parsed_target) = buf.to_export_params();
        assert_eq!(parsed_path, path);
        assert_eq!(parsed_target, target_bytes);
    }

    #[test]
    fn test_to_export_ptrs() {
        let mut buf = Buffer::new();
        let path = "/tmp/export";
        let target_bytes: u32 = 512;

        // Setup buffer layout: [path\0][target_bytes aligned][ptrs aligned]
        for (i, &b) in path.as_bytes().iter().enumerate() {
            buf.data[i] = b;
        }
        buf.data[path.len()] = 0;
        
        // Write target_bytes at aligned offset
        let target_aligned = Buffer::align_up(path.len() + 1, std::mem::align_of::<u32>());
        unsafe {
            *(buf.data.as_mut_ptr().add(target_aligned) as *mut u32) = target_bytes;
        }
        
        // Write ptrs at aligned offset after target_bytes
        let ptrs_aligned = Buffer::align_up(target_aligned + 4, std::mem::align_of::<AssetPtr>());
        let ptr1 = AssetPtr::new(0, 100);
        let ptr2 = AssetPtr::new(1, 200);
        let ptrs_arr = [ptr1, ptr2];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ptrs_arr.as_ptr(),
                buf.data.as_mut_ptr().add(ptrs_aligned) as *mut AssetPtr,
                2
            );
        }

        let ptrs = buf.to_export_ptrs(2);
        assert_eq!(ptrs.len(), 2);
        assert_eq!(ptrs[0], ptr1);
        assert_eq!(ptrs[1], ptr2);
    }

    #[test]
    fn test_to_import_paths() {
        let mut buf = Buffer::new();
        let paths = ["/tmp/batch_000.elbo", "/tmp/batch_001.elbo"];
        
        let mut offset = 0;
        for path in &paths {
            for (i, &b) in path.as_bytes().iter().enumerate() {
                buf.data[offset + i] = b;
            }
            buf.data[offset + path.len()] = 0;
            offset += path.len() + 1;
        }

        let result = buf.to_import_paths(2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], paths[0]);
        assert_eq!(result[1], paths[1]);
    }

    #[test]
    fn test_to_export_response() {
        let mut buf = Buffer::new();
        let filenames = ["batch_000.elbo", "batch_001.elbo", "batch_002.elbo"];
        
        // Write count
        let count = filenames.len() as u32;
        buf.data[0..4].copy_from_slice(&count.to_le_bytes());
        
        // Write null-terminated filenames
        let mut offset = 4;
        for filename in &filenames {
            for (i, &b) in filename.as_bytes().iter().enumerate() {
                buf.data[offset + i] = b;
            }
            buf.data[offset + filename.len()] = 0;
            offset += filename.len() + 1;
        }

        let result = buf.to_export_response();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], filenames[0]);
        assert_eq!(result[1], filenames[1]);
        assert_eq!(result[2], filenames[2]);
    }

    #[test]
    fn test_to_import_paths_empty() {
        let buf = Buffer::new();
        let result = buf.to_import_paths(0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_to_export_response_empty() {
        let mut buf = Buffer::new();
        buf.data[0..4].copy_from_slice(&0u32.to_le_bytes());
        let result = buf.to_export_response();
        assert!(result.is_empty());
    }
}
