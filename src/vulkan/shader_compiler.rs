//! Runtime shader compilation module
//! 
//! This module provides independent shader compilation capabilities,
//! allowing the application to compile GLSL shaders to SPIR-V at runtime
//! without depending on external tools.

use shaderc::Compiler;
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::error::{Result, VulkanError};
use log::{debug, info, error};

/// Shader cache entry containing compiled SPIR-V bytecode
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Compiled SPIR-V bytecode
    spirv: Vec<u32>,
    /// Source file modification time
    #[allow(dead_code)]
    source_modified: std::time::SystemTime,
    /// Compilation timestamp
    #[allow(dead_code)]
    compiled_at: std::time::SystemTime,
}

/// Runtime shader compiler with caching capabilities
pub struct ShaderCompiler {
    /// Shaderc compiler instance
    compiler: Compiler,
    /// Cache for compiled shaders
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    /// Enable/disable shader caching
    enable_cache: bool,
    /// Enable/disable debug info in compiled shaders
    enable_debug: bool,
    /// Optimization level for compilation
    optimization_level: shaderc::OptimizationLevel,
}

impl ShaderCompiler {
    /// Create a new shader compiler
    /// 
    /// # Returns
    /// A new ShaderCompiler instance
    /// 
    /// # Errors
    /// Returns an error if compiler initialization fails
    pub fn new() -> Result<Self> {
        info!("Initializing shader compiler");
        
        let compiler = Compiler::new().or_else(|_| {
            Err(VulkanError::ShaderCompilation("Failed to create shader compiler".to_string()))
        })?;
        
        info!("Shader compiler initialized successfully");
        
        Ok(Self {
            compiler,
            cache: Arc::new(Mutex::new(HashMap::new())),
            enable_cache: true,
            enable_debug: cfg!(debug_assertions),
            optimization_level: if cfg!(debug_assertions) {
                shaderc::OptimizationLevel::Zero
            } else {
                shaderc::OptimizationLevel::Performance
            },
        })
    }
    
    /// Configure shader compilation settings
    /// 
    /// # Arguments
    /// * `enable_cache` - Whether to enable shader caching
    /// * `enable_debug` - Whether to include debug information
    /// * `optimization_level` - Compilation optimization level
    pub fn configure(&mut self, enable_cache: bool, enable_debug: bool, optimization_level: shaderc::OptimizationLevel) {
        self.enable_cache = enable_cache;
        self.enable_debug = enable_debug;
        self.optimization_level = optimization_level;
        
        debug!("Shader compiler configured: cache={}, debug={}, opt={:?}", 
               enable_cache, enable_debug, optimization_level);
    }
    
    /// Compile a GLSL shader file to SPIR-V
    /// 
    /// # Arguments
    /// * `shader_path` - Path to the GLSL shader file
    /// * `entry_point` - Entry point function name (usually "main")
    /// 
    /// # Returns
    /// Compiled SPIR-V bytecode as Vec<u32>
    /// 
    /// # Errors
    /// Returns an error if compilation fails
    pub fn compile_file(&mut self, shader_path: &str, entry_point: &str) -> Result<Vec<u32>> {
        let shader_path = Path::new(shader_path);
        
        // Determine shader kind from file extension
        let shader_kind = self.determine_shader_kind(shader_path)?;
        
        // Read shader source
        let source = fs::read_to_string(shader_path)
            .map_err(|e| VulkanError::ShaderCompilation(format!("Failed to read shader file '{}': {}", shader_path.display(), e)))?;
        
        // Compile the shader
        self.compile_source(&source, shader_path.to_str().unwrap(), entry_point, shader_kind)
    }
    
    /// Compile GLSL source code to SPIR-V
    /// 
    /// # Arguments
    /// * `source` - GLSL source code
    /// * `file_name` - File name for error reporting
    /// * `entry_point` - Entry point function name
    /// * `kind` - Shader type (vertex, fragment, etc.)
    /// 
    /// # Returns
    /// Compiled SPIR-V bytecode as Vec<u32>
    /// 
    /// # Errors
    /// Returns an error if compilation fails
    pub fn compile_source(&mut self, source: &str, file_name: &str, entry_point: &str, kind: shaderc::ShaderKind) -> Result<Vec<u32>> {
        debug!("Compiling shader '{}' with entry point '{}'", file_name, entry_point);
        
        // Check cache first if enabled
        if self.enable_cache {
            if let Some(cached_spirv) = self.check_cache(source, file_name) {
                info!("Using cached compiled shader: {}", file_name);
                return Ok(cached_spirv);
            }
        }
        
        // Compile the shader
        let mut compile_options = shaderc::CompileOptions::new().or_else(|_| {
            Err(VulkanError::ShaderCompilation("Failed to create compile options".to_string()))
        })?;
        
        // Set optimization level
        compile_options.set_optimization_level(self.optimization_level);
        
        // Enable debug info in debug builds
        if self.enable_debug {
            compile_options.set_generate_debug_info();
            debug!("Debug info enabled for shader compilation");
        }
        
        // Set target environment to Vulkan 1.0
        compile_options.set_target_env(shaderc::TargetEnv::Vulkan, shaderc::EnvVersion::Vulkan1_0 as u32);
        
        // Compile the shader
        let artifact = self.compiler
            .compile_into_spirv(source, kind, file_name, entry_point, Some(&compile_options))
            .map_err(|e| {
                error!("Shader compilation failed for '{}': {}", file_name, e);
                VulkanError::ShaderCompilation(format!("Failed to compile shader '{}': {}", file_name, e))
            })?;
        
        // Get the compiled SPIR-V
        let spirv = artifact.as_binary().to_vec();
        
        if spirv.is_empty() {
            return Err(VulkanError::ShaderCompilation(
                format!("Compilation produced empty SPIR-V for shader '{}'", file_name)
            ).into());
        }
        
        info!("Shader '{}' compiled successfully ({} words)", file_name, spirv.len());
        debug!("Shader '{}' optimization level: {:?}", file_name, self.optimization_level);
        
        // Cache the result if enabled
        if self.enable_cache {
            self.cache_result(file_name, &spirv);
        }
        
        Ok(spirv)
    }
    
    /// Determine shader kind from file extension
    /// 
    /// # Arguments
    /// * `path` - File path
    /// 
    /// # Returns
    /// ShaderKind for the file
    /// 
    /// # Errors
    /// Returns an error if the file extension is not recognized
    fn determine_shader_kind(&self, path: &Path) -> Result<shaderc::ShaderKind> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| VulkanError::ShaderCompilation(
                format!("No file extension found for shader: {}", path.display())
            ))?;
        
        match extension.to_lowercase().as_str() {
            "vert" => Ok(shaderc::ShaderKind::Vertex),
            "frag" => Ok(shaderc::ShaderKind::Fragment),
            "geom" => Ok(shaderc::ShaderKind::Geometry),
            "comp" => Ok(shaderc::ShaderKind::Compute),
            "tesc" => Ok(shaderc::ShaderKind::TessControl),
            "tese" => Ok(shaderc::ShaderKind::TessEvaluation),
            _ => Err(VulkanError::ShaderCompilation(
                format!("Unsupported shader extension: {}", extension)
            ).into()),
        }
    }
    
    /// Check if a cached version of the shader is available and valid
    /// 
    /// # Arguments
    /// * `source` - Current shader source code
    /// * `file_name` - File name for cache key
    /// 
    /// # Returns
    /// Cached SPIR-V if available and valid, None otherwise
    fn check_cache(&self, source: &str, file_name: &str) -> Option<Vec<u32>> {
        let cache = self.cache.lock().unwrap();
        
        if let Some(entry) = cache.get(file_name) {
            // Simple cache validation: compare source length and modification time
            // In a more sophisticated implementation, we could hash the source content
            let _current_time = std::time::SystemTime::now();
            
            // Use the source content hash for more accurate cache validation
            let source_hash = self.hash_source(source);
            let cached_hash = self.hash_source(&format!("{:?}", entry.spirv));
            
            if source_hash == cached_hash {
                debug!("Cache hit for shader: {}", file_name);
                return Some(entry.spirv.clone());
            } else {
                debug!("Cache miss for shader: {} (source changed)", file_name);
            }
        } else {
            debug!("Cache miss for shader: {} (not cached)", file_name);
        }
        
        None
    }
    
    /// Cache compilation result
    /// 
    /// # Arguments
    /// * `file_name` - File name for cache key
    /// * `spirv` - Compiled SPIR-V bytecode
    fn cache_result(&self, file_name: &str, spirv: &[u32]) {
        let mut cache = self.cache.lock().unwrap();
        
        let entry = CacheEntry {
            spirv: spirv.to_vec(),
            source_modified: std::time::SystemTime::now(),
            compiled_at: std::time::SystemTime::now(),
        };
        
        cache.insert(file_name.to_string(), entry);
        debug!("Cached compiled shader: {}", file_name);
    }
    
    /// Simple hash function for source content validation
    /// 
    /// # Arguments
    /// * `content` - Content to hash
    /// 
    /// # Returns
    /// Simple hash value
    fn hash_source(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Clear the shader cache
    #[allow(dead_code)]
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        info!("Shader cache cleared");
    }
    
    /// Get cache statistics
    /// 
    /// # Returns
    /// Tuple of (number of cached shaders, total cached size in bytes)
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().unwrap();
        let count = cache.len();
        let size: usize = cache.values()
            .map(|entry| entry.spirv.len() * std::mem::size_of::<u32>())
            .sum();
        
        (count, size)
    }
    
    /// Preload and compile commonly used shaders
    /// 
    /// # Arguments
    /// * `shader_paths` - List of shader file paths to preload
    /// 
    /// # Errors
    /// Returns an error if any shader fails to compile
    pub fn preload_shaders(&mut self, shader_paths: &[&str]) -> Result<()> {
        info!("Preloading {} shaders", shader_paths.len());
        
        for &shader_path in shader_paths {
            debug!("Preloading shader: {}", shader_path);
            self.compile_file(shader_path, "main")?;
        }
        
        info!("Shader preloading completed successfully");
        Ok(())
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create default shader compiler")
    }
}

impl Drop for ShaderCompiler {
    fn drop(&mut self) {
        let (count, size) = self.get_cache_stats();
        if count > 0 {
            info!("Shader compiler dropped: {} cached shaders ({} bytes)", count, size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shader_compiler_creation() {
        let compiler = ShaderCompiler::new();
        assert!(compiler.is_ok());
    }
    
    #[test]
    fn test_shader_kind_determination() {
        let compiler = ShaderCompiler::new().unwrap();
        
        assert!(matches!(
            compiler.determine_shader_kind(Path::new("test.vert")).unwrap(),
            shaderc::ShaderKind::Vertex
        ));
        assert!(matches!(
            compiler.determine_shader_kind(Path::new("test.frag")).unwrap(),
            shaderc::ShaderKind::Fragment
        ));
        
        assert!(compiler.determine_shader_kind(Path::new("test.unknown")).is_err());
    }
    
    #[test]
    fn test_cache_operations() {
        let compiler = ShaderCompiler::new().unwrap();
        
        // Initially empty cache
        let (count, size) = compiler.get_cache_stats();
        assert_eq!(count, 0);
        assert_eq!(size, 0);
        
        // Clear cache should not fail
        compiler.clear_cache();
        
        let (count, size) = compiler.get_cache_stats();
        assert_eq!(count, 0);
        assert_eq!(size, 0);
    }
    
    #[test]
    fn test_source_hashing() {
        let compiler = ShaderCompiler::new().unwrap();
        
        let source1 = "void main() {}";
        let source2 = "void main() {}";
        let source3 = "void main() { int x; }";
        
        let hash1 = compiler.hash_source(source1);
        let hash2 = compiler.hash_source(source2);
        let hash3 = compiler.hash_source(source3);
        
        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
    }
}
