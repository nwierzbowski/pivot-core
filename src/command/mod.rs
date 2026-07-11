mod engine_command;
mod engine_response;
mod mesh_publish;

use iceoryx2::prelude::ZeroCopySend;

use crate::{
    Buffer, MAX_HANDLE_LEN,
};

// Bring in impl blocks from submodules (structs defined here, impls there)
#[allow(unused_imports)]
use engine_command as _;
#[allow(unused_imports)]
use engine_response as _;
#[allow(unused_imports)]
use mesh_publish as _;

#[cfg(test)]
mod tests;

// ============================================================================
// Operation IDs
// ============================================================================

pub const OP_STANDARDIZE_GROUPS: u16 = 1;
pub const OP_STANDARDIZE_SYNCED_GROUPS: u16 = 2;
pub const OP_SET_SURFACE_TYPES: u16 = 3;
pub const OP_DROP_GROUPS: u16 = 4;
pub const OP_ORGANIZE_OBJECTS: u16 = 5;
pub const OP_GET_SURFACE_TYPES: u16 = 6;
pub const OP_STOP_ENGINE: u16 = 7;
pub const OP_ALLOC_MEM: u16 = 8;
pub const OP_SEND_MESH: u16 = 10;
pub const OP_EXPORT_ASSETS: u16 = 11;
pub const OP_IMPORT_ASSETS: u16 = 12;
pub const OP_EXPORT_ALL: u16 = 13;
pub const OP_DROP_ALL_GROUPS: u16 = 15;
pub const OP_TBO_POINTS_FLUSH: u16 = 17;
pub const OP_EXPORT_ASSET_TBO: u16 = 18;
pub const OP_EXPORT_ALL_ASSET_TBO: u16 = 19;
pub const OP_GROUP_ALL_OBJECTS: u16 = 20;
pub const OP_EMBED_ALL_ASSETS: u16 = 21;

// ============================================================================
// Wire types
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, iceoryx2::prelude::ZeroCopySend)]
pub struct EngineCommand {
    pub should_cache: u16,
    pub op_id: u16,
    pub num_headers: u32,
    pub inline_data: Buffer,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, iceoryx2::prelude::ZeroCopySend)]
pub struct ResponseHeader {
    pub status: u16,
    pub total_slabs: u16,
    pub num_items: u32,
    pub root_slab_handle: [u8; MAX_HANDLE_LEN],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, iceoryx2::prelude::ZeroCopySend)]
pub struct EngineResponse {
    pub header: ResponseHeader,
    pub inline_data: Buffer,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, iceoryx2::prelude::ZeroCopySend)]
pub struct MeshPublish {
    pub header: ResponseHeader,
    pub inline_data: Buffer,
}
