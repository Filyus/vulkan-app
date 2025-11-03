use legion::{World, Resources, IntoQuery};
use crate::ecs::components::{Transform, Mesh, Renderable, Triangle, Color, Vertex};
use cgmath::Vector3;

pub fn create_triangle_mesh(world: &mut World, resources: &mut Resources) {
    let mut triangle_entities = resources.get_mut::<Vec<legion::Entity>>()
        .expect("Triangle entities vector not found in resources");
    
    // Create triangle vertices with rainbow colors
    let vertices = vec![
        Vertex {
            position: Vector3::new(0.0, 0.5, 0.0),
            color: Vector3::new(1.0, 0.0, 0.0), // Red
        },
        Vertex {
            position: Vector3::new(-0.5, -0.5, 0.0),
            color: Vector3::new(0.0, 1.0, 0.0), // Green
        },
        Vertex {
            position: Vector3::new(0.5, -0.5, 0.0),
            color: Vector3::new(0.0, 0.0, 1.0), // Blue
        },
    ];

    let indices = vec![0, 1, 2];

    let entity = world.push((
        Triangle,
        Mesh {
            vertices,
            indices,
        },
        Transform::default(),
        Renderable {
            vertex_count: 3,
            index_count: 3,
        },
    ));

    triangle_entities.push(entity);
}

pub fn render_system(world: &mut World, resources: &mut Resources) {
    let mut vulkan_renderer = resources.get_mut::<crate::vulkan::renderer::VulkanRenderer>()
        .expect("VulkanRenderer resource not found - make sure it's properly initialized");
    let mut query = <(&Mesh, &Transform, &Renderable)>::query();
    
    // This system will handle rendering entities
    // For now, we'll just collect the render data
    let render_data: Vec<_> = query.iter(world).collect();
    
    if !render_data.is_empty() {
        // Update the renderer with the latest mesh data
        if let Some((mesh, _transform, _renderable)) = render_data.first() {
            vulkan_renderer.update_vertices(&mesh.vertices);
            vulkan_renderer.update_indices(&mesh.indices);
        }
    }
}

pub fn transform_update_system(world: &mut World, _resources: &mut Resources) {
    let mut query = <(&mut Transform, &Color)>::query();
    
    // This system could update transforms over time
    // For now, it's a placeholder for future animation logic
    for (_transform, _color) in query.iter_mut(world) {
        // Example: Rotate entities slowly
        // transform.rotation.z += 0.01;
    }
}