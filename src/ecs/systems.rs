use legion::{World, Resources, IntoQuery};
use crate::ecs::components::{Transform, Mesh, Renderable, Triangle, Color, Vertex};
use crate::error::{Result, EcsError};
use cgmath::Vector3;
use log::{debug, info, warn};

/// Create a triangle mesh entity in the ECS world
///
/// # Arguments
/// * `world` - The ECS world to add the entity to
/// * `resources` - The resources container
///
/// # Returns
/// * Ok(()) if the triangle was created successfully
/// * Err if creation failed
pub fn create_triangle_mesh(world: &mut World, resources: &mut Resources) -> Result<()> {
    let mut triangle_entities = resources.get_mut::<Vec<legion::Entity>>()
        .ok_or_else(|| EcsError::ResourceAccess("Triangle entities vector not found in resources".to_string()))?;
    
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
    
    info!("Created triangle mesh entity");
    debug!("Triangle entity ID: {:?}", entity);
    
    Ok(())
}

/// System that handles rendering entities
///
/// This system collects render data from entities and updates the Vulkan renderer
///
/// # Arguments
/// * `world` - The ECS world containing entities
/// * `resources` - The resources container including the Vulkan renderer
pub fn render_system(world: &mut World, resources: &mut Resources) {
    let mut vulkan_renderer = match resources.get_mut::<crate::vulkan::renderer::VulkanRenderer>() {
        Some(renderer) => renderer,
        None => {
            warn!("VulkanRenderer resource not found in render system");
            return;
        }
    };
    
    let mut query = <(&Mesh, &Transform, &Renderable)>::query();
    
    // Collect all renderable entities
    let render_data: Vec<_> = query.iter(world).collect();
    
    if render_data.is_empty() {
        debug!("No renderable entities found");
        return;
    }
    
    debug!("Rendering {} entities", render_data.len());
    
    // Update the renderer with the latest mesh data
    // For now, we'll just use the first mesh
    if let Some((mesh, transform, _renderable)) = render_data.first() {
        debug!("Updating renderer with mesh data ({} vertices, {} indices)",
               mesh.vertices.len(), mesh.indices.len());
        
        vulkan_renderer.update_vertices(&mesh.vertices);
        vulkan_renderer.update_indices(&mesh.indices);
        
        // Log transform information for debugging
        debug!("Entity transform: position={:?}, rotation={:?}, scale={:?}",
               transform.position, transform.rotation, transform.scale);
    }
}

/// System that updates entity transforms over time
///
/// This system can be used to animate entities by updating their transforms
///
/// # Arguments
/// * `world` - The ECS world containing entities
/// * `resources` - The resources container
pub fn transform_update_system(world: &mut World, _resources: &mut Resources) {
    let mut query = <(&mut Transform, &Color)>::query();
    
    let mut entity_count = 0;
    
    // This system could update transforms over time
    // For now, it's a placeholder for future animation logic
    for (transform, color) in query.iter_mut(world) {
        entity_count += 1;
        
        // Example: Rotate entities slowly based on their color
        // This is just an application of how to access component data
        if color.r > 0.5 {
            // Red entities rotate faster
            // transform.rotation.z += 0.02;
        } else if color.g > 0.5 {
            // Green entities rotate at medium speed
            // transform.rotation.z += 0.01;
        } else {
            // Blue entities rotate slower
            // transform.rotation.z += 0.005;
        }
        
        // Log transform information for debugging
        debug!("Transform update for entity: position={:?}, rotation={:?}, scale={:?}",
               transform.position, transform.rotation, transform.scale);
    }
    
    if entity_count > 0 {
        debug!("Updated transforms for {} entities", entity_count);
    }
}

/// System that logs statistics about the ECS world
///
/// This system can be used for debugging and monitoring the ECS state
///
/// # Arguments
/// * `world` - The ECS world containing entities
/// * `resources` - The resources container
#[allow(dead_code)] // For future ECS debugging
pub fn debug_system(world: &mut World, resources: &mut Resources) {
    // This is a debug system that can be used to monitor ECS state
    let triangle_count = <(&Triangle,)>::query().iter(world).count();
    let mesh_count = <(&Mesh,)>::query().iter(world).count();
    let transform_count = <(&Transform,)>::query().iter(world).count();
    let renderable_count = <(&Renderable,)>::query().iter(world).count();
    
    info!("ECS Debug Info:");
    info!("  Triangle entities: {}", triangle_count);
    info!("  Mesh components: {}", mesh_count);
    info!("  Transform components: {}", transform_count);
    info!("  Renderable components: {}", renderable_count);
    
    // Log resource information
    if let Some(triangle_entities) = resources.get::<Vec<legion::Entity>>() {
        info!("  Tracked triangle entities: {}", triangle_entities.len());
    }
}