use cgmath::Vector3;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Renderable {
    pub vertex_count: u32,
    pub index_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle;

#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// SDF Components

#[derive(Clone, Debug, PartialEq)]
pub enum SDFShapeType {
    Sphere,
    Box,
    #[allow(dead_code)]
    Plane,
    #[allow(dead_code)]
    Torus,
    #[allow(dead_code)]
    Cylinder,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SDFShape {
    pub shape_type: SDFShapeType,
    pub size: f32,
    pub params: [f32; 4], // Additional parameters for complex shapes
}

impl Default for SDFShape {
    fn default() -> Self {
        Self {
            shape_type: SDFShapeType::Sphere,
            size: 1.0,
            params: [0.0; 4],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SDFMaterial {
    pub color: cgmath::Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: f32,
}

impl Default for SDFMaterial {
    fn default() -> Self {
        Self {
            color: cgmath::Vector3::new(1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            emission: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SDFRenderable;

#[derive(Clone, Debug, PartialEq)]
pub struct SDFLight {
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector3<f32>,
    pub intensity: f32,
}

impl Default for SDFLight {
    fn default() -> Self {
        Self {
            position: cgmath::Vector3::new(2.0, 2.0, 2.0),
            color: cgmath::Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_new() {
        let pos = Vector3::new(1.0, 2.0, 3.0);
        let color = Vector3::new(0.5, 0.7, 0.9);
        let vertex = Vertex { position: pos, color };
        
        assert_eq!(vertex.position, pos);
        assert_eq!(vertex.color, color);
    }

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        let expected_pos = Vector3::new(0.0, 0.0, 0.0);
        let expected_rot = Vector3::new(0.0, 0.0, 0.0);
        let expected_scale = Vector3::new(1.0, 1.0, 1.0);
        
        assert_eq!(transform.position, expected_pos);
        assert_eq!(transform.rotation, expected_rot);
        assert_eq!(transform.scale, expected_scale);
    }

    #[test]
    fn test_transform_new() {
        let pos = Vector3::new(1.0, 2.0, 3.0);
        let rot = Vector3::new(0.5, 1.0, 1.5);
        let scale = Vector3::new(2.0, 3.0, 4.0);
        let transform = Transform { position: pos, rotation: rot, scale };
        
        assert_eq!(transform.position, pos);
        assert_eq!(transform.rotation, rot);
        assert_eq!(transform.scale, scale);
    }

    #[test]
    fn test_mesh_new() {
        let vertices = vec![
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color: Vector3::new(1.0, 0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                color: Vector3::new(0.0, 1.0, 0.0)
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                color: Vector3::new(0.0, 0.0, 1.0)
            }
        ];
        let indices = vec![0, 1, 2];
        let mesh = Mesh { vertices: vertices.clone(), indices: indices.clone() };
        
        assert_eq!(mesh.vertices, vertices);
        assert_eq!(mesh.indices, indices);
    }

    #[test]
    fn test_renderable_new() {
        let renderable = Renderable { vertex_count: 3, index_count: 3 };
        assert_eq!(renderable.vertex_count, 3);
        assert_eq!(renderable.index_count, 3);
    }

    #[test]
    fn test_color_new() {
        let color = Color { r: 0.5, g: 0.7, b: 0.9 };
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.7);
        assert_eq!(color.b, 0.9);
    }

    #[test]
    fn test_component_equality() {
        let pos1 = Vector3::new(1.0, 2.0, 3.0);
        let pos2 = Vector3::new(1.0, 2.0, 3.0);
        let pos3 = Vector3::new(1.0, 2.0, 4.0);
        
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_transform_equality() {
        let transform1 = Transform::default();
        let transform2 = Transform::default();
        let transform3 = Transform {
            position: Vector3::new(1.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };
        
        assert_eq!(transform1, transform2);
        assert_ne!(transform1, transform3);
    }

    #[test]
    fn test_mesh_equality() {
        let vertices = vec![
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color: Vector3::new(1.0, 0.0, 0.0)
            }
        ];
        let indices = vec![0];
        
        let mesh1 = Mesh { vertices: vertices.clone(), indices: indices.clone() };
        let mesh2 = Mesh { vertices: vertices.clone(), indices: indices.clone() };
        let mesh3 = Mesh { vertices: vertices.clone(), indices: vec![1] };
        
        assert_eq!(mesh1, mesh2);
        assert_ne!(mesh1, mesh3);
    }
}