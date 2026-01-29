use iceoryx2::prelude::*;

use crate::{Buffer};

pub const OP_STANDARDIZE_GROUPS: u16 = 1;
pub const OP_STANDARDIZE_SYNCED_GROUPS: u16 = 2;
pub const OP_SET_SURFACE_TYPES: u16 = 3;
pub const OP_DROP_GROUPS: u16 = 4;
pub const OP_ORGANIZE_OBJECTS: u16 = 5;
pub const OP_GET_SURFACE_TYPES: u16 = 6;
pub const OP_STOP_ENGINE: u16 = 7;


#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct EngineCommand {
    pub should_cache: u16,
    pub op_id: u16,
    pub num_groups: u32,
    pub inline_data: Buffer,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct EngineResponse {
    pub status: u32, // 0 for OK, 1 for Error, etc.
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct MeshPublish {
    pub num_groups: u64,
    pub inline_data: Buffer,
}


