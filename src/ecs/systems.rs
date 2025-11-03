use legion::{World, Resources, IntoQuery};
use crate::ecs::components::{
    Transform, Mesh, Renderable, Triangle, Color, Vertex,
    SDFShape, SDFMaterial, SDFRenderable, SDFLight, SDFShapeType
};
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn render_system(world: &mut World, resources: &mut Resources) {
    let _vulkan_renderer = match resources.get_mut::<crate::vulkan::renderer::VulkanRenderer>() {
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
        debug!("Rendering mesh data ({} vertices, {} indices)",
               mesh.vertices.len(), mesh.indices.len());
        
        // Note: The renderer now uses SDF shaders instead of traditional vertex rendering
        // The mesh data is logged for debugging but not directly used by the renderer
        
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

/// Create SDF entities in the ECS world
///
/// # Arguments
/// * `world` - The ECS world to add entities to
/// * `resources` - The resources container
///
/// # Returns
/// * Ok(()) if the SDF entities were created successfully
/// * Err if creation failed
pub fn create_sdf_entities(world: &mut World, resources: &mut Resources) -> Result<()> {
    let mut sdf_entities = resources.get_mut::<Vec<legion::Entity>>()
        .ok_or_else(|| EcsError::ResourceAccess("SDF entities vector not found in resources".to_string()))?;
    
    // Create a red sphere at center
    let sphere_entity = world.push((
        SDFShape {
            shape_type: SDFShapeType::Sphere,
            size: 0.5,
            params: [0.0; 4],
        },
        SDFMaterial {
            color: Vector3::new(1.0, 0.0, 0.0),
            metallic: 0.0,
            roughness: 0.5,
            emission: 0.0,
        },
        Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        SDFRenderable,
    ));
    
    // Create a green box on the left
    let box_entity = world.push((
        SDFShape {
            shape_type: SDFShapeType::Box,
            size: 0.3,
            params: [0.0; 4],
        },
        SDFMaterial {
            color: Vector3::new(0.0, 1.0, 0.0),
            metallic: 0.1,
            roughness: 0.7,
            emission: 0.0,
        },
        Transform {
            position: Vector3::new(-1.5, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        SDFRenderable,
    ));
    
    // Create a blue sphere on the right
    let sphere2_entity = world.push((
        SDFShape {
            shape_type: SDFShapeType::Sphere,
            size: 0.4,
            params: [0.0; 4],
        },
        SDFMaterial {
            color: Vector3::new(0.0, 0.0, 1.0),
            metallic: 0.3,
            roughness: 0.3,
            emission: 0.0,
        },
        Transform {
            position: Vector3::new(1.5, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        SDFRenderable,
    ));
    
    // Create a light
    let light_entity = world.push((
        SDFLight {
            position: Vector3::new(2.0, 2.0, 2.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        },
    ));
    
    sdf_entities.push(sphere_entity);
    sdf_entities.push(box_entity);
    sdf_entities.push(sphere2_entity);
    sdf_entities.push(light_entity);
    
    info!("Created SDF entities: sphere, box, sphere, and light");
    debug!("SDF entity IDs: sphere={:?}, box={:?}, sphere2={:?}, light={:?}",
           sphere_entity, box_entity, sphere2_entity, light_entity);
    
    Ok(())
}

/// System that handles SDF rendering
///
/// This system collects SDF render data from entities and updates the Vulkan renderer
///
/// # Arguments
/// * `world` - The ECS world containing entities
/// * `resources` - The resources container including the Vulkan renderer
pub fn sdf_render_system(world: &mut World, resources: &mut Resources) {
    let _vulkan_renderer = match resources.get_mut::<crate::vulkan::renderer::VulkanRenderer>() {
        Some(renderer) => renderer,
        None => {
            warn!("VulkanRenderer resource not found in SDF render system");
            return;
        }
    };
    
    let mut sdf_query = <(&SDFShape, &SDFMaterial, &Transform)>::query();
    let mut light_query = <&SDFLight>::query();
    
    // Collect all SDF renderable entities
    let sdf_entities: Vec<_> = sdf_query.iter(world).collect();
    let lights: Vec<_> = light_query.iter(world).collect();
    
    if sdf_entities.is_empty() {
        debug!("No SDF renderable entities found");
        return;
    }
    
    debug!("Rendering {} SDF entities with {} lights", sdf_entities.len(), lights.len());
    
    // For now, the SDF data is hardcoded in the shader
    // In a future implementation, we would update uniform buffers with ECS data
    for (shape, material, transform) in sdf_entities {
        debug!("SDF entity: shape={:?}, size={}, position={:?}, color={:?}",
               shape.shape_type, shape.size, transform.position, material.color);
    }
    
    for light in lights {
        debug!("Light: position={:?}, color={:?}, intensity={}",
               light.position, light.color, light.intensity);
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