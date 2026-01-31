use bytemuck::{Pod, Zeroable};



#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable, PartialEq, Eq, Hash)]
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
impl<'py> pyo3::FromPyObject<'py> for Uuid {
    fn extract_bound(ob: &pyo3::Bound<'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let bytes: [u8; 16] = ob.extract()?;
        Ok(Self { bytes })
    }
}