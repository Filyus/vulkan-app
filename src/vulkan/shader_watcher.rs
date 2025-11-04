//! Hot shader reload functionality
//!
//! This module provides file watching capabilities for automatic shader
//! recompilation and pipeline recreation when shader files change.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::collections::{HashMap, VecDeque};
use notify::{Watcher, RecursiveMode, Event, RecommendedWatcher};
use log::{info, error, debug, warn};
use crate::error::{Result, VulkanError};
use crate::vulkan::shader_compiler::ShaderCompiler;
use crate::vulkan::pipeline::VulkanPipeline;
use crate::config;

/// Shader change event callback type
pub type ShaderChangeCallback = Box<dyn Fn(&str, &str) -> Result<()> + Send + Sync>;

/// Hot reload configuration
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Enable/disable hot reload
    pub enabled: bool,
    /// Shader directory to watch
    pub shader_dir: PathBuf,
    /// Debounce time for file changes (milliseconds)
    pub debounce_ms: u64,
    /// File extensions to watch
    pub watch_extensions: Vec<String>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: config::hot_reload::ENABLED,
            shader_dir: PathBuf::from(config::hot_reload::SHADER_DIR),
            debounce_ms: config::hot_reload::DEBOUNCE_MS,
            watch_extensions: config::hot_reload::WATCH_EXTENSIONS.iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Shader file watcher for hot reload
pub struct ShaderWatcher {
    /// File system watcher
    _watcher: RecommendedWatcher,
    /// Hot reload configuration
    config: HotReloadConfig,
    /// Map of file paths to last modification times
    file_times: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
    /// Shader compiler reference
    #[allow(dead_code)]
    shader_compiler: Arc<Mutex<ShaderCompiler>>,
    /// Callback for shader changes
    #[allow(dead_code)]
    change_callback: Option<ShaderChangeCallback>,
    /// Whether the watcher is running
    #[allow(dead_code)]
    is_running: Arc<Mutex<bool>>,
    /// Arc reference to callback for thread-safe access
    _change_callback_arc: Arc<Mutex<Option<ShaderChangeCallback>>>,
}

impl ShaderWatcher {
    /// Create a new shader watcher
    /// 
    /// # Arguments
    /// * `config` - Hot reload configuration
    /// * `shader_compiler` - Shader compiler instance
    /// 
    /// # Returns
    /// A new ShaderWatcher instance
    /// 
    /// # Errors
    /// Returns an error if watcher creation fails
    pub fn new(config: HotReloadConfig, shader_compiler: Arc<Mutex<ShaderCompiler>>) -> Result<Self> {
        info!("Creating shader watcher with config: {:?}", config);
        
        let file_times = Arc::new(Mutex::new(HashMap::new()));
        let is_running = Arc::new(Mutex::new(false));
        let change_callback = Arc::new(Mutex::new(None::<ShaderChangeCallback>));
        
        // Clone the Arcs for the watcher thread
        let file_times_clone = Arc::clone(&file_times);
        let config_clone = config.clone();
        let is_running_clone = Arc::clone(&is_running);
        let change_callback_clone = Arc::clone(&change_callback);
        
        // Create the file system watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        let callback_guard = change_callback_clone.lock().unwrap();
                        if let Err(e) = Self::handle_file_event(event, &config_clone, &file_times_clone, &is_running_clone, &*callback_guard) {
                            error!("Error handling file event: {}", e);
                        }
                    }
                    Err(e) => error!("File watcher error: {:?}", e),
                }
            },
            notify::Config::default(),
        ).map_err(|e| VulkanError::ShaderCompilation(format!("Failed to create file watcher: {}", e)))?;
        
        // Start watching the shader directory
        if config.enabled {
            watcher.watch(&config.shader_dir, RecursiveMode::Recursive)
                .map_err(|e| VulkanError::ShaderCompilation(format!("Failed to watch shader directory: {}", e)))?;
            
            info!("Started watching shader directory: {:?}", config.shader_dir);
            
            // Initialize file times for existing shader files
            Self::initialize_file_times(&config.shader_dir, &file_times, &config.watch_extensions)?;
        }
        
        Ok(Self {
            _watcher: watcher,
            config,
            file_times,
            shader_compiler,
            change_callback: None,
            is_running,
            _change_callback_arc: change_callback,
        })
    }
    
    /// Set the callback for shader changes
    /// 
    /// # Arguments
    /// * `callback` - Callback function to call when shaders change
    pub fn set_change_callback(&mut self, callback: ShaderChangeCallback) {
        // Update the Arc for thread-safe access first
        if let Ok(mut callback_arc) = self._change_callback_arc.lock() {
            *callback_arc = Some(callback);
        }
        // Don't store in self.change_callback since we moved it to the Arc
    }
    
    /// Enable or disable hot reload
    /// 
    /// # Arguments
    /// * `enabled` - Whether to enable hot reload
    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        if enabled == self.config.enabled {
            return Ok(());
        }
        
        self.config.enabled = enabled;
        
        if enabled {
            info!("Enabling hot shader reload");
            // Start watching
            self._watcher.watch(&self.config.shader_dir, RecursiveMode::Recursive)
                .map_err(|e| VulkanError::ShaderCompilation(format!("Failed to watch shader directory: {}", e)))?;
            
            // Initialize file times
            Self::initialize_file_times(&self.config.shader_dir, &self.file_times, &self.config.watch_extensions)?;
        } else {
            info!("Disabling hot shader reload");
            // Stop watching
            let _ = self._watcher.unwatch(&self.config.shader_dir);
        }
        
        Ok(())
    }

    /// Check if hot reload is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Handle file system events
    fn handle_file_event(
        event: Event,
        config: &HotReloadConfig,
        file_times: &Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
        is_running: &Arc<Mutex<bool>>,
        callback: &Option<ShaderChangeCallback>,
    ) -> Result<()> {
        if !config.enabled {
            return Ok(());
        }
        
        // Prevent concurrent processing
        {
            let mut running = is_running.lock().unwrap();
            if *running {
                debug!("File event processing already in progress, skipping");
                return Ok(());
            }
            *running = true;
        }
        
        let result = Self::process_file_event(event, config, file_times, callback);
        
        // Clear the running flag
        {
            let mut running = is_running.lock().unwrap();
            *running = false;
        }
        
        result
    }
    
    /// Process a single file event
    fn process_file_event(
        event: Event,
        config: &HotReloadConfig,
        file_times: &Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
        callback: &Option<ShaderChangeCallback>,
    ) -> Result<()> {
        debug!("File event: {:?}", event);
        
        for path in event.paths {
            // Check if the file has a shader extension we care about
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if !config.watch_extensions.contains(&extension.to_string()) {
                    continue;
                }
                
                // Get the current modification time
                let metadata = std::fs::metadata(&path);
                if let Ok(metadata) = metadata {
                    let current_time = metadata.modified()
                        .unwrap_or_else(|_| SystemTime::now());
                    
                    // Check if we should process this file
                    let should_process = {
                        let times = file_times.lock().unwrap();
                        if let Some(last_time) = times.get(&path) {
                            // Only process if the file is newer than our last record
                            current_time.duration_since(*last_time).unwrap_or(Duration::ZERO) >= Duration::from_millis(config.debounce_ms)
                        } else {
                            // New file, always process
                            true
                        }
                    };
                    
                    if should_process {
                        if config::hot_reload::LOG_RELOAD_EVENTS {
                            info!("Shader file changed: {:?}", path);
                        }
                        
                        // Update the last modification time
                        {
                            let mut times = file_times.lock().unwrap();
                            times.insert(path.clone(), current_time);
                        }
                        
                        // Trigger shader reload
                        if let Some(shader_path) = path.to_str() {
                            // Determine shader kind from extension
                            let shader_kind = match extension {
                                "vert" => "vertex",
                                "frag" => "fragment",
                                "geom" => "geometry",
                                "comp" => "compute",
                                "tesc" => "tess_control",
                                "tese" => "tess_evaluation",
                                _ => "unknown",
                            };
                            
                            // Check if this shader type should be reloaded
                            let should_reload = match shader_kind {
                                "vertex" => config::hot_reload::RELOAD_VERTEX_SHADERS,
                                "fragment" => config::hot_reload::RELOAD_FRAGMENT_SHADERS,
                                "geometry" => config::hot_reload::RELOAD_GEOMETRY_SHADERS,
                                "compute" => config::hot_reload::RELOAD_COMPUTE_SHADERS,
                                "tess_control" | "tess_evaluation" => config::hot_reload::RELOAD_TESSELLATION_SHADERS,
                                _ => false,
                            };
                            
                            if should_reload {
                                if config::hot_reload::LOG_RELOAD_EVENTS {
                                    info!("Triggering hot reload for {} shader: {}", shader_kind, shader_path);
                                }
                                debug!("Hot reload triggered for: {} ({})", shader_path, shader_kind);
                                
                                // Actually trigger the callback to handle the shader change
                                if let Some(ref callback) = callback {
                                    if let Err(e) = callback(shader_path, shader_kind) {
                                        error!("Failed to handle shader change: {}", e);
                                    }
                                }
                            } else {
                                debug!("Skipping reload for disabled shader type: {} ({})", shader_path, shader_kind);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Initialize file modification times for existing shader files
    fn initialize_file_times(
        shader_dir: &Path,
        file_times: &Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
        watch_extensions: &[String],
    ) -> Result<()> {
        debug!("Initializing file times for shader directory: {:?}", shader_dir);
        
        if !shader_dir.exists() {
            warn!("Shader directory does not exist: {:?}", shader_dir);
            return Ok(());
        }
        
        let mut times = file_times.lock().unwrap();
        
        for entry in std::fs::read_dir(shader_dir)
            .map_err(|e| VulkanError::ShaderCompilation(format!("Failed to read shader directory: {}", e)))? 
        {
            let entry = entry.map_err(|e| VulkanError::ShaderCompilation(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            // Check if it's a file with a shader extension
            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    if watch_extensions.contains(&extension.to_string()) {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            if let Ok(modified_time) = metadata.modified() {
                                times.insert(path.clone(), modified_time);
                                debug!("Initialized file time for: {:?}", path);
                            }
                        }
                    }
                }
            }
        }
        
        info!("Initialized file times for {} shader files", times.len());
        Ok(())
    }

    /// Manually trigger a reload for a specific shader file
    ///
    /// # Arguments
    /// * `shader_path` - Path to the shader file to reload
    #[allow(dead_code)]
    pub fn reload_shader(&self, shader_path: &str) -> Result<()> {
        info!("Manual reload requested for shader: {}", shader_path);
        
        if let Some(callback) = &self.change_callback {
            // Determine shader kind from file extension
            let path = Path::new(shader_path);
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| VulkanError::ShaderCompilation(
                    format!("No file extension found for shader: {}", shader_path)
                ))?;
            
            let shader_kind = match extension {
                "vert" => "vertex",
                "frag" => "fragment",
                "geom" => "geometry",
                "comp" => "compute",
                "tesc" => "tess_control",
                "tese" => "tess_evaluation",
                _ => "unknown",
            };
            
            // Check if this shader type should be reloaded
            let should_reload = match shader_kind {
                "vertex" => config::hot_reload::RELOAD_VERTEX_SHADERS,
                "fragment" => config::hot_reload::RELOAD_FRAGMENT_SHADERS,
                "geometry" => config::hot_reload::RELOAD_GEOMETRY_SHADERS,
                "compute" => config::hot_reload::RELOAD_COMPUTE_SHADERS,
                "tess_control" | "tess_evaluation" => config::hot_reload::RELOAD_TESSELLATION_SHADERS,
                _ => false,
            };
            
            if should_reload {
                callback(shader_path, shader_kind)?;
                info!("Manual reload completed for: {}", shader_path);
            } else {
                warn!("Manual reload skipped for disabled shader type: {} ({})", shader_path, shader_kind);
            }
        } else {
            warn!("No change callback set for shader reload");
        }
        
        Ok(())
    }
    
    /// Get statistics about the watcher
    #[allow(dead_code)]
    pub fn get_stats(&self) -> (usize, bool) {
        let file_count = self.file_times.lock().unwrap().len();
        let is_enabled = self.config.enabled;
        (file_count, is_enabled)
    }
}

impl Drop for ShaderWatcher {
    fn drop(&mut self) {
        info!("Shader watcher dropped");
    }
}

/// Pending shader reload request
#[derive(Debug)]
pub struct ShaderReloadRequest {
    /// Path to the shader file to reload
    pub shader_path: String,
    /// Type of shader (vertex, fragment, etc.)
    pub shader_kind: String,
}

/// Hot reload manager that coordinates shader watching and pipeline recreation
pub struct HotReloadManager {
    /// Shader watcher
    watcher: Option<ShaderWatcher>,
    /// Configuration
    config: HotReloadConfig,
    /// Shader compiler
    shader_compiler: Arc<Mutex<ShaderCompiler>>,
    /// Pipeline reference for recreation
    pipeline: Option<Arc<Mutex<VulkanPipeline>>>,
    /// Queue of pending reload requests to be processed in main thread
    pending_reloads: Arc<Mutex<VecDeque<ShaderReloadRequest>>>,
    /// Flag to track if reloads occurred in the current frame
    reloads_occurred: Arc<Mutex<bool>>,
}

impl HotReloadManager {
    /// Create a new hot reload manager
    ///
    /// # Arguments
    /// * `config` - Hot reload configuration
    /// * `shader_compiler` - Shader compiler instance
    ///
    /// # Returns
    /// A new HotReloadManager instance
    pub fn new(config: HotReloadConfig, shader_compiler: Arc<Mutex<ShaderCompiler>>) -> Self {
        Self {
            watcher: None,
            config,
            shader_compiler,
            pipeline: None,
            pending_reloads: Arc::new(Mutex::new(VecDeque::new())),
            reloads_occurred: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Initialize the hot reload manager
    ///
    /// # Arguments
    /// * `pipeline` - Vulkan pipeline to recreate when shaders change
    ///
    /// # Returns
    /// Ok(()) if initialization succeeded
    ///
    /// # Errors
    /// Returns an error if initialization fails
    pub fn initialize(&mut self, pipeline: Arc<Mutex<VulkanPipeline>>) -> Result<()> {
        info!("Initializing hot reload manager");

        self.pipeline = Some(pipeline);

        if self.config.enabled {
            let shader_compiler = Arc::clone(&self.shader_compiler);
            let _pipeline_ref = self.pipeline.clone().unwrap();

            // Create the watcher with a callback
            let mut watcher = ShaderWatcher::new(self.config.clone(), shader_compiler)?;

            // Set up the change callback to queue reload requests instead of immediate processing
            let pending_reloads_clone = Arc::clone(&self.pending_reloads);

            watcher.set_change_callback(Box::new(move |shader_path: &str, shader_kind: &str| {
                Self::queue_shader_reload(shader_path, shader_kind, &pending_reloads_clone)
            }));

            self.watcher = Some(watcher);
            info!("Hot reload manager initialized successfully");
        } else {
            info!("Hot reload disabled in configuration");
        }

        Ok(())
    }

    /// Queue a shader reload request for later processing
    fn queue_shader_reload(
        shader_path: &str,
        shader_kind: &str,
        pending_reloads: &Arc<Mutex<VecDeque<ShaderReloadRequest>>>,
    ) -> Result<()> {
        if config::hot_reload::LOG_RELOAD_EVENTS {
            info!("Queueing shader reload for: {} ({})", shader_path, shader_kind);
        }

        let request = ShaderReloadRequest {
            shader_path: shader_path.to_string(),
            shader_kind: shader_kind.to_string(),
        };

        // Add to pending reloads queue
        let mut queue = pending_reloads.lock().unwrap();
        queue.push_back(request);

        // Keep only the most recent reloads to prevent queue buildup
        if queue.len() > 10 {
            let _old = queue.pop_front();
            warn!("Dropped old shader reload request to prevent queue buildup");
        }

        debug!("Shader reload queued. Queue length: {}", queue.len());
        Ok(())
    }

    /// Process all pending shader reload requests safely
    /// This should be called from the main render thread when it's safe to recreate pipelines
    ///
    /// # Returns
    /// * Ok(true) if pipeline was recreated and command buffers need updating
    /// * Ok(false) if no pipeline recreation occurred
    /// * Err if processing failed
    pub fn process_pending_reloads(&self) -> Result<bool> {
        let mut queue = self.pending_reloads.lock().unwrap();
        if queue.is_empty() {
            return Ok(false);
        }

        let reloads_to_process: Vec<ShaderReloadRequest> = queue.drain(..).collect();
        drop(queue); // Release lock before processing

        info!("=== PROCESSING PENDING SHADER RELOADS ===");
        info!("Processing {} pending shader reload requests", reloads_to_process.len());

        let mut pipeline_recreated = false;

        if let Some(ref pipeline) = self.pipeline {
            for request in reloads_to_process {
                info!("Processing reload for: {} ({})", request.shader_path, request.shader_kind);

                // CRITICAL: Recreate the pipeline with the new shader
                // This will involve proper GPU synchronization
                {
                    let mut pipeline_guard = pipeline.lock().unwrap();
                    if let Err(e) = pipeline_guard.recompile_shader(&request.shader_path) {
                        error!("FAILED to recreate pipeline for {}: {}", request.shader_path, e);
                        // Continue processing other reloads even if one fails
                    } else {
                        info!("SUCCESS: Pipeline recreated for: {}", request.shader_path);
                        pipeline_recreated = true;
                    }
                }
            }
        } else {
            warn!("No pipeline available for shader reload");
        }

        // Set the reloads occurred flag to true for backward compatibility
        {
            let mut reloads_flag = self.reloads_occurred.lock().unwrap();
            *reloads_flag = true;
            debug!("Set reloads_occurred flag to true");
        }

        if pipeline_recreated {
            info!("=== SHADER RELOAD COMPLETED - COMMAND BUFFERS MUST BE UPDATED IMMEDIATELY ===");
        } else {
            info!("=== SHADER RELOAD COMPLETED - NO PIPELINE CHANGES ===");
        }

        Ok(pipeline_recreated)
    }

    /// Check if reloads occurred and clear the flag
    pub fn check_and_clear_reloads_occurred(&self) -> bool {
        let mut reloads_flag = self.reloads_occurred.lock().unwrap();
        let occurred = *reloads_flag;
        debug!("check_and_clear_reloads_occurred: flag was {}", occurred);
        *reloads_flag = false;
        occurred
    }

    /// Get the number of pending reload requests
    #[allow(dead_code)]
    pub fn pending_reload_count(&self) -> usize {
        self.pending_reloads.lock().unwrap().len()
    }
    
    /// Enable or disable hot reload
    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.config.enabled = enabled;
        
        if let Some(ref mut watcher) = self.watcher {
            watcher.set_enabled(enabled)?;
        }
        
        info!("Hot reload {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }
    
    /// Check if hot reload is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get hot reload statistics
    #[allow(dead_code)]
    pub fn get_stats(&self) -> (usize, bool) {
        if let Some(ref watcher) = self.watcher {
            watcher.get_stats()
        } else {
            (0, false)
        }
    }
    
    /// Manually trigger a shader reload (queues it for safe processing)
    pub fn reload_shader(&self, shader_path: &str) -> Result<()> {
        if let Some(extension) = Path::new(shader_path).extension().and_then(|ext| ext.to_str()) {
            let shader_kind = match extension {
                "vert" => "vertex",
                "frag" => "fragment",
                "geom" => "geometry",
                "comp" => "compute",
                "tesc" => "tess_control",
                "tese" => "tess_evaluation",
                _ => "unknown",
            };

            // Queue the reload request instead of processing immediately
            Self::queue_shader_reload(shader_path, shader_kind, &self.pending_reloads)
        } else {
            warn!("Invalid shader path: {}", shader_path);
            Ok(())
        }
    }
}

impl Drop for HotReloadManager {
    fn drop(&mut self) {
        info!("Hot reload manager dropped");

        // Explicitly release pipeline reference to break reference cycle
        if let Some(pipeline_arc) = self.pipeline.take() {
            debug!("Releasing pipeline reference from hot reload manager");
            // The pipeline will be cleaned up when all Arc references are dropped
            drop(pipeline_arc);
        }

        // Shader watcher will be automatically dropped and stopped
        debug!("Shader watcher will be cleaned up automatically");
    }
}
