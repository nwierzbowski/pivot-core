#[repr(C)]
pub struct AssetPtr {
    packed_ptr: u64,
}

impl AssetPtr {
    pub fn new(slab_index: u16, offset: u64) -> Self {
        let packed_ptr = ((slab_index as u64) << 48) | (offset & 0x0000_FFFF_FFFF_FFFF);
        Self { packed_ptr }
    }

    pub fn unpack(&self) -> (u16, u64) {
        let slab_index = (self.packed_ptr >> 48) as u16;
        let offset = self.packed_ptr & 0x0000_FFFF_FFFF_FFFF;
        (slab_index, offset)
    }
}