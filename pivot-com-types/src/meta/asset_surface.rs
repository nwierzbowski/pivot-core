use iceoryx2::prelude::*;

use crate::MAX_NAME_LEN;

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct GroupSurface {
    pub group_name: [u8; MAX_NAME_LEN],
    pub surface_type: u64, // "wood", "metal", etc.
}

impl GroupSurface {
    pub fn new(group_name: &str, surface_type: u64) -> Self {
        let mut surf = GroupSurface {
            group_name: [0; MAX_NAME_LEN],
            surface_type,
        };

        surf.set_group_name(group_name);
        surf
    }

    fn set_group_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(MAX_NAME_LEN - 1);
        self.group_name[..len].copy_from_slice(&bytes[..len]);
    }
}