use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Point3D; 3],
}

impl Triangle {
    pub fn normal(&self) -> Point3D {
        todo!("Implementation needed: Calculate triangle normal")
    }

    pub fn area(&self) -> f32 {
        todo!("Implementation needed: Calculate triangle area")
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub points: Vec<Point2D>,
}

impl Polygon {
    pub fn contains_point(&self, point: Point2D) -> bool {
        todo!("Implementation needed: Point-in-polygon test")
    }

    pub fn area(&self) -> f32 {
        todo!("Implementation needed: Calculate polygon area")
    }
}

