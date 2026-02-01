use bytemuck::{Pod, Zeroable};



#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable, PartialEq, Eq, Hash, Debug)]
pub struct Uuid {
    bytes: [u8; 16],
}

pub struct Vert {
    x: f32,
    y: f32,
    z: f32,
}

pub struct Edge {
    v1: u32,
    v2: u32,
}

pub struct Matrix4x4 {
    data: [f32; 16],
}

#[cfg(feature = "pyo3")]
impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for Uuid {
    type Error = pyo3::PyErr;

    fn extract(ob: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> Result<Self, Self::Error> {
        let bytes: [u8; 16] = ob.extract()?;
        Ok(Self { bytes })
    }
}