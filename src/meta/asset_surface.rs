use iceoryx2::prelude::*;

use crate::{fields::Uuid};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
pub struct GroupSurface {
    pub uuid: Uuid,
    pub surface_type: u64, // "wood", "metal", etc.
}

impl GroupSurface {
    pub fn new(uuid: Uuid, surface_type: u64) -> Self {
        let surf = GroupSurface {
            uuid,
            surface_type,
        };
        surf
    }
}