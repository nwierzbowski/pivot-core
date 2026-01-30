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