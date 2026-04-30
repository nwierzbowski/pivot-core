use bytemuck::{Pod, Zeroable};
use iceoryx2::prelude::ZeroCopySend;



#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, PartialEq, Eq, Hash, Debug, ZeroCopySend)]
pub struct Uuid {
    pub bytes: [u8; 32],
}

impl Uuid {
    pub const SIZE: usize = 32;
}

#[cfg(feature = "pyo3")]
impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for Uuid {
    type Error = pyo3::PyErr;

    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; Uuid::SIZE];
        let extracted: Vec<u8> = ob.extract()?;
        let len = extracted.len().min(Uuid::SIZE);
        bytes[..len].copy_from_slice(&extracted[..len]);
        Ok(Self { bytes })
    }
}
