use crate::fields::Uuid;

pub struct AllocRequestMeta {
    pub num_assets: u64,
    pub offset_uuids: usize,
    pub offset_sizes: usize,
}

impl AllocRequestMeta {
    pub fn new(num_assets: u64) -> Self {
        let count = num_assets as usize;

        let mut cursor = size_of::<Self>();

        let offset_uuids = cursor;
        cursor = align_to_8(offset_uuids + (count * size_of::<Uuid>()));

        let offset_sizes = cursor;
        // cursor = align_to_8(offset_sizes + (count * size_of::<usize>()));

        // let total_size = cursor;

        Self {
            num_assets,
            offset_uuids,
            offset_sizes,
        }
    }
}

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
        cursor = align_to_8(offset_uuids + (count * size_of::<Uuid>()));

        let offset_packed_ptrs = cursor;
        // cursor = align_to_8(offset_packed_ptrs + (count * size_of::<u64>()));

        // let total_size = cursor;

        Self {
            num_assets,
            offset_uuids,
            offset_packed_ptrs,
        }
    }
}

fn align_to_8(val: usize) -> usize {
    (val + 7) & !7
}
