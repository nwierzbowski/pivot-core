use iceoryx2::prelude::*;

use crate::MAX_NAME_LEN;

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub struct GroupNames {
    pub group_name: [u8; MAX_NAME_LEN],
}

impl GroupNames {
    pub fn new(group_name: &str) -> Self {
        let mut gn = GroupNames {
            group_name: [0; MAX_NAME_LEN],
        };
        let bytes = group_name.as_bytes();
        let len = bytes.len().min(MAX_NAME_LEN - 1);
        gn.group_name[..len].copy_from_slice(&bytes[..len]);
        gn
    }
}