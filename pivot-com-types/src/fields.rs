use bytemuck::{Pod, Zeroable};
use iceoryx2::prelude::ZeroCopySend;



#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, PartialEq, Eq, Hash, Debug, ZeroCopySend)]
pub struct Uuid {
    bytes: [u8; 16],
}

#[cfg(feature = "pyo3")]
impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for Uuid {
    type Error = pyo3::PyErr;

    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> Result<Self, Self::Error> {
        let bytes: [u8; 16] = ob.extract()?;
        Ok(Self { bytes })
    }
}