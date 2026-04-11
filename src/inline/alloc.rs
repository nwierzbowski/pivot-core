use bytemuck::{Pod, Zeroable};

use crate::{MAX_HANDLE_LEN, fields::Uuid};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct AllocRequestMeta {
    pub num_assets: u64,
    pub offset_uuids: usize,
    pub offset_sizes: usize,
}


fn align_to_32(val: usize) -> usize {
    (val + 31) & !31
}

impl AllocRequestMeta {
    pub fn new(num_assets: u64) -> Self {
        let count = num_assets as usize;

        let mut cursor = size_of::<Self>();

        let offset_uuids = cursor;
        cursor = align_to_32(offset_uuids + (count * size_of::<Uuid>()));

        let offset_sizes = cursor;
        // cursor = align_to_32(offset_sizes + (count * size_of::<usize>()));

        // let total_size = cursor;

        Self {
            num_assets,
            offset_uuids,
            offset_sizes,
        }
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]

pub struct SlabRegistry {
    pub num_slabs: u64,
    pub slab_handles: [[u8; MAX_HANDLE_LEN]; 64],
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct AllocResponseMeta {
    pub num_assets: u64,
    pub offset_uuids: usize,
    pub offset_packed_ptrs: usize,
}

impl AllocResponseMeta {
    pub fn new(num_assets: u64) -> Self {
        let count = num_assets as usize;

        let mut cursor = size_of::<Self>();

        let offset_uuids = cursor;
        cursor = align_to_32(offset_uuids + (count * size_of::<Uuid>()));

        let offset_packed_ptrs = cursor;
        // cursor = align_to_32(offset_packed_ptrs + (count * size_of::<u64>()));

        // let total_size = cursor;

        Self {
            num_assets,
            offset_uuids,
            offset_packed_ptrs,
        }
    }
}
