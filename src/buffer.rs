use bytemuck::{Pod, Zeroable};
use iceoryx2::prelude::*;

use crate::{asset_ptr::AssetPtr, asset_surface::GroupSurface, fields::Uuid};

pub const MAX_INLINE_DATA: usize = 65536; // 64 KB (L1 Cache Friendly)

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferError {
    Overflow,
    Corrupted,
    MismatchedLengths,
    InvalidUtf8,
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::Overflow => write!(f, "Data exceeds buffer capacity"),
            BufferError::Corrupted => write!(f, "Corrupted or invalid buffer data"),
            BufferError::MismatchedLengths => write!(f, "Mismatched slice lengths"),
            BufferError::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
        }
    }
}

impl std::error::Error for BufferError {}

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

    /// Align offset up to the nearest multiple of `align`
    pub fn align_up(offset: usize, align: usize) -> usize {
        (offset + align - 1) & !(align - 1)
    }

    /// Raw pointer access to internal data (for command constructors)
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Raw const pointer access to internal data
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Mutable slice access to internal data
    #[inline]
    pub fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Const slice access to internal data
    #[inline]
    pub fn as_ref(&self) -> &[u8] {
        &self.data
    }

    // ============================================================================
    // Writers - low-level, type-agnostic
    // ============================================================================

    /// Write a slice of any Copy type to a specific offset
    pub fn write_slice<T: Copy>(&mut self, data: &[T], offset: usize) -> Result<(), BufferError> {
        let size = data.len().checked_mul(std::mem::size_of::<T>())
            .ok_or(BufferError::Overflow)?;
        if offset.checked_add(size).ok_or(BufferError::Overflow)? > MAX_INLINE_DATA {
            return Err(BufferError::Overflow);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                self.data.as_mut_ptr().add(offset),
                size,
            );
        }
        Ok(())
    }

    /// Write UUID slice at offset 0
    pub fn write_uuids(&mut self, uuids: &[Uuid]) -> Result<(), BufferError> {
        self.write_slice(uuids, 0)
    }

    /// Write AssetPtr slice at offset 0
    pub fn write_asset_ptrs(&mut self, ptrs: &[AssetPtr]) -> Result<(), BufferError> {
        self.write_slice(ptrs, 0)
    }

    /// Write GroupSurface slice at offset 0
    pub fn write_group_surfaces(&mut self, surfaces: &[GroupSurface]) -> Result<(), BufferError> {
        self.write_slice(surfaces, 0)
    }

    // ============================================================================
    // Readers - low-level, type-agnostic
    // ============================================================================

    /// Read UUIDs from offset 0
    pub fn read_uuids(&self, count: usize) -> Result<&[Uuid], BufferError> {
        let required = count.checked_mul(std::mem::size_of::<Uuid>()).ok_or(BufferError::Overflow)?;
        if required > MAX_INLINE_DATA {
            return Err(BufferError::Overflow);
        }
        unsafe {
            Ok(std::slice::from_raw_parts(self.data.as_ptr() as *const Uuid, count))
        }
    }

    /// Read AssetPtrs from offset 0
    pub fn read_asset_ptrs(&self, count: usize) -> Result<&[AssetPtr], BufferError> {
        let required = count.checked_mul(std::mem::size_of::<AssetPtr>()).ok_or(BufferError::Overflow)?;
        if required > MAX_INLINE_DATA {
            return Err(BufferError::Overflow);
        }
        unsafe {
            Ok(std::slice::from_raw_parts(self.data.as_ptr() as *const AssetPtr, count))
        }
    }

    /// Read GroupSurfaces from offset 0
    pub fn read_group_surfaces(&self, count: usize) -> Result<&[GroupSurface], BufferError> {
        let required = count.checked_mul(std::mem::size_of::<GroupSurface>()).ok_or(BufferError::Overflow)?;
        if required > MAX_INLINE_DATA {
            return Err(BufferError::Overflow);
        }
        unsafe {
            Ok(std::slice::from_raw_parts(self.data.as_ptr() as *const GroupSurface, count))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_uuid(id: u8) -> Uuid {
        let mut bytes = [0u8; 16];
        bytes[0] = id;
        Uuid { bytes }
    }

    #[test]
    fn test_write_read_uuids() {
        let mut buf = Buffer::new();
        let uuids = [make_uuid(1), make_uuid(2), make_uuid(3)];
        buf.write_uuids(&uuids).unwrap();
        let result = buf.read_uuids(3).unwrap();
        assert_eq!(result, &uuids);
    }

    #[test]
    fn test_write_read_asset_ptrs() {
        let mut buf = Buffer::new();
        let ptrs = [AssetPtr::new(0, 100), AssetPtr::new(1, 200)];
        buf.write_asset_ptrs(&ptrs).unwrap();
        let result = buf.read_asset_ptrs(2).unwrap();
        assert_eq!(result, &ptrs);
    }

    #[test]
    fn test_write_read_group_surfaces() {
        let mut buf = Buffer::new();
        let surfaces = [GroupSurface::new(make_uuid(1), 1), GroupSurface::new(make_uuid(2), 2)];
        buf.write_group_surfaces(&surfaces).unwrap();
        let result = buf.read_group_surfaces(2).unwrap();
        assert_eq!(result, &surfaces);
    }

    #[test]
    fn test_overflow_detection() {
        let mut buf = Buffer::new();
        // 10000 * 16 bytes = 160KB > 64KB
        let large_slice = vec![make_uuid(0); 10000];
        assert!(buf.write_uuids(&large_slice).is_err());
    }
}
