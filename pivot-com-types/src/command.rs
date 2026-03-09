use iceoryx2::prelude::*;

use crate::{Buffer, MAX_HANDLE_LEN};

pub const OP_STANDARDIZE_GROUPS: u16 = 1;
pub const OP_STANDARDIZE_SYNCED_GROUPS: u16 = 2;
pub const OP_SET_SURFACE_TYPES: u16 = 3;
pub const OP_DROP_GROUPS: u16 = 4;
pub const OP_ORGANIZE_OBJECTS: u16 = 5;
pub const OP_GET_SURFACE_TYPES: u16 = 6;
pub const OP_STOP_ENGINE: u16 = 7;
pub const OP_ALLOC_MEM: u16 = 8;
pub const OP_EXTRACT_GEOMETRIC_FEATURES: u16 = 9;
pub const OP_SEND_MESH: u16 = 10;


#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct EngineCommand {
    pub should_cache: u16,
    pub op_id: u16,
    pub num_headers: u32,
    pub inline_data: Buffer,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct ResponseHeader {
    pub status: u16,
    pub total_slabs: u16,
    pub num_items: u32,
    pub root_slab_handle: [u8; MAX_HANDLE_LEN],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct EngineResponse {
    pub header: ResponseHeader,
    pub inline_data: Buffer,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct MeshPublish {
    pub header: ResponseHeader,
    pub inline_data: Buffer,
}


