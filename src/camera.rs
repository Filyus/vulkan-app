//! Camera module for proper 3D projection and aspect ratio handling
//!
//! This module provides a robust camera system that correctly handles aspect ratio
//! and projection for 3D rendering, preventing stretching during window resize.

use cgmath::{Vector3, Matrix4, Point3, Rad, Deg, perspective, InnerSpace};

/// Camera structure for 3D rendering with proper aspect ratio handling
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world space
    pub position: Point3<f32>,
    
    /// Camera target (what we're looking at)
    pub target: Point3<f32>,
    
    /// Up vector (world up direction)
    pub up: Vector3<f32>,
    
    /// Field of view in radians
    pub fovy: Rad<f32>,
    
    /// Near plane distance
    pub near: f32,
    
    /// Far plane distance
    pub far: f32,
    
    /// Aspect ratio (width/height)
    pub aspect_ratio: f32,
    
    /// Cached view matrix
    view_matrix: Matrix4<f32>,
    
    /// Cached projection matrix
    projection_matrix: Matrix4<f32>,
    
    /// Cached view-projection matrix
    view_projection_matrix: Matrix4<f32>,
}

impl Camera {
    /// Create a new camera with default settings
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut camera = Self {
            position: Point3::new(0.0, 0.0, 2.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            fovy: Deg(45.0).into(),
            near: 0.1,
            far: 100.0,
            aspect_ratio: 1.0,
            view_matrix: Matrix4::from_scale(1.0),
            projection_matrix: Matrix4::from_scale(1.0),
            view_projection_matrix: Matrix4::from_scale(1.0),
        };
        camera.update_matrices();
        camera
    }
    
    /// Create a new camera with specific parameters
    pub fn with_params(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        fovy: Rad<f32>,
        near: f32,
        far: f32,
        aspect_ratio: f32,
    ) -> Self {
        let mut camera = Self {
            position,
            target,
            up,
            fovy,
            near,
            far,
            aspect_ratio,
            view_matrix: Matrix4::from_scale(1.0),
            projection_matrix: Matrix4::from_scale(1.0),
            view_projection_matrix: Matrix4::from_scale(1.0),
        };
        camera.update_matrices();
        camera
    }
    
    /// Set camera position
    #[allow(dead_code)]
    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
        self.update_matrices();
    }
    
    /// Set Camera target
    #[allow(dead_code)]
    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
        self.update_matrices();
    }
    
    /// Set Camera up vector
    #[allow(dead_code)]
    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up;
        self.update_matrices();
    }
    
    /// Set field of view
    #[allow(dead_code)]
    pub fn set_fovy(&mut self, fovy: Rad<f32>) {
        self.fovy = fovy;
        self.update_matrices();
    }
    
    /// Set near and far planes
    #[allow(dead_code)]
    pub fn set_near_far(&mut self, near: f32, far: f32) {
        self.near = near;
        self.far = far;
        self.update_matrices();
    }
    
    /// Set aspect ratio (for window resize)
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.update_matrices();
    }
    
    /// Update all cached matrices
    pub fn update_matrices(&mut self) {
        self.view_matrix = self.calculate_view_matrix();
        self.projection_matrix = self.calculate_projection_matrix();
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
    }
    
    /// Calculate the view matrix (look-at matrix)
    fn calculate_view_matrix(&self) -> Matrix4<f32> {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward);
        
        Matrix4::look_at_rh(self.position, self.target, up)
    }
    
    /// Calculate the projection matrix with proper aspect ratio handling
    fn calculate_projection_matrix(&self) -> Matrix4<f32> {
        // Create perspective projection with correct aspect ratio
        perspective(
            self.fovy,
            self.aspect_ratio,
            self.near,
            self.far
        )
    }
    
    /// Get the view matrix
    #[allow(dead_code)]
    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }
    
    /// Get the projection matrix
    #[allow(dead_code)]
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
    }
    
    /// Get the combined view-projection matrix
    #[allow(dead_code)]
    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.view_projection_matrix
    }
    
    /// Get the forward vector
    #[allow(dead_code)]
    pub fn forward(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }
    
    /// Get the right vector
    #[allow(dead_code)]
    pub fn right(&self) -> Vector3<f32> {
        let forward = self.forward();
        forward.cross(self.up).normalize()
    }
    
    /// Get the up vector
    #[allow(dead_code)]
    pub fn up(&self) -> Vector3<f32> {
        self.up
    }
}

/// Utility functions for camera calculations
pub mod utils {
    use super::*;
    
    /// Create a ray direction from screen coordinates
    /// This properly handles aspect ratio for ray marching
    #[allow(dead_code)]
    pub fn screen_to_ray(
        screen_x: f32,
        screen_y: f32,
        aspect_ratio: f32,
    ) -> Vector3<f32> {
        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let ndc_x = screen_x * 2.0 - 1.0;
        let ndc_y = screen_y * 2.0 - 1.0;
        
        // Apply aspect ratio correction to x coordinate
        // This ensures proper field of view without stretching
        let corrected_x = ndc_x * aspect_ratio;
        
        // Create ray direction in camera space
        Vector3::new(corrected_x, ndc_y, 1.0).normalize()
    }
    
    /// Create a view matrix for a camera looking at a target
    #[allow(dead_code)]
    pub fn create_look_at_matrix(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
    ) -> Matrix4<f32> {
        Matrix4::look_at_rh(eye, target, up)
    }
}