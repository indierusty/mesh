pub type Id = usize;

pub enum Handle {
    Linear,
    Quadratic(Vec2),
    Cubic(Vec2, Vec2),
}

pub struct Vec2(f64, f64);

pub enum MeshElement {
    Point(Id, Vec2),
    Segment(Id, Id, Handle, Id),
}

pub struct Mesh {
    elements: Vec<MeshElement>,
}

fn main() {
    println!("Hello, world!");
}
