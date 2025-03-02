use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{FoldDbError, FoldDbResult};
use crate::datafold_node::app::manifest::AppManifest;
use crate::datafold_node::app::registry::AppRegistry;
use crate::datafold_node::app::resource::{AppResourceManager, ResourceAllocation};

/// Manages loading apps from disk
pub struct AppLoader {
    /// Base directory for apps
    base_dir: PathBuf,
    
    /// App registry
    registry: AppRegistry,
    
    /// Resource manager
    resource_manager: AppResourceManager,
}

impl AppLoader {
    /// Creates a new app loader
    pub fn new(base_dir: &Path, registry: AppRegistry, resource_manager: AppResourceManager) -> Self {
        Self {
            base_dir: base_dir.to_path_buf(),
            registry,
            resource_manager,
        }
    }
    
    /// Loads an app from a directory
    pub fn load_app(&mut self, app_dir: &Path) -> FoldDbResult<()> {
        // Check if directory exists
        if !app_dir.exists() || !app_dir.is_dir() {
            return Err(FoldDbError::Config(format!("App directory '{}' does not exist or is not a directory", app_dir.display())));
        }
        
        // Load manifest
        let manifest_path = app_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(FoldDbError::Config(format!("App manifest '{}' does not exist", manifest_path.display())));
        }
        
        // Read manifest
        let manifest_str = fs::read_to_string(&manifest_path)
            .map_err(|e| FoldDbError::Config(format!("Failed to read app manifest: {}", e)))?;
        
        // Parse manifest
        let manifest: AppManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| FoldDbError::Config(format!("Failed to parse app manifest: {}", e)))?;
        
        // Validate manifest
        manifest.validate()
            .map_err(|e| FoldDbError::Config(format!("Invalid app manifest: {}", e)))?;
        
        // Register app
        self.registry.register_app(manifest.clone())?;
        
        // Allocate resources
        let resources = ResourceAllocation {
            memory: 50 * 1024 * 1024, // 50 MB
            cpu: 25.0,                // 25% of one CPU core
            storage: 5 * 1024 * 1024, // 5 MB
            bandwidth: 512 * 1024,    // 512 KB/s
        };
        
        self.resource_manager.allocate_resources(&manifest.name, resources)?;
        
        Ok(())
    }
    
    /// Loads all apps from the base directory
    pub fn load_all_apps(&mut self) -> FoldDbResult<Vec<String>> {
        // Check if base directory exists
        if !self.base_dir.exists() || !self.base_dir.is_dir() {
            return Err(FoldDbError::Config(format!("Base directory '{}' does not exist or is not a directory", self.base_dir.display())));
        }
        
        // Get app directories
        let entries = fs::read_dir(&self.base_dir)
            .map_err(|e| FoldDbError::Config(format!("Failed to read base directory: {}", e)))?;
        
        let mut loaded_apps = Vec::new();
        
        // Load each app
        for entry in entries {
            let entry = entry.map_err(|e| FoldDbError::Config(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Check if directory contains a manifest
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    // Load app
                    if let Err(e) = self.load_app(&path) {
                        eprintln!("Failed to load app from directory '{}': {}", path.display(), e);
                    } else {
                        // Get app name from directory name
                        if let Some(app_name) = path.file_name().and_then(|n| n.to_str()) {
                            loaded_apps.push(app_name.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(loaded_apps)
    }
    
    /// Loads an app from a manifest file
    pub fn load_app_from_manifest(&mut self, manifest_path: &Path) -> FoldDbResult<()> {
        // Check if manifest exists
        if !manifest_path.exists() {
            return Err(FoldDbError::Config(format!("App manifest '{}' does not exist", manifest_path.display())));
        }
        
        // Read manifest
        let manifest_str = fs::read_to_string(manifest_path)
            .map_err(|e| FoldDbError::Config(format!("Failed to read app manifest: {}", e)))?;
        
        // Parse manifest
        let manifest: AppManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| FoldDbError::Config(format!("Failed to parse app manifest: {}", e)))?;
        
        // Validate manifest
        manifest.validate()
            .map_err(|e| FoldDbError::Config(format!("Invalid app manifest: {}", e)))?;
        
        // Register app
        self.registry.register_app(manifest.clone())?;
        
        // Allocate resources
        let resources = ResourceAllocation {
            memory: 50 * 1024 * 1024, // 50 MB
            cpu: 25.0,                // 25% of one CPU core
            storage: 5 * 1024 * 1024, // 5 MB
            bandwidth: 512 * 1024,    // 512 KB/s
        };
        
        self.resource_manager.allocate_resources(&manifest.name, resources)?;
        
        Ok(())
    }
    
    /// Unloads an app
    pub fn unload_app(&mut self, app_name: &str) -> FoldDbResult<()> {
        // Release resources
        self.resource_manager.release_resources(app_name)?;
        
        // Unregister app
        self.registry.unregister_app(app_name)?;
        
        Ok(())
    }
}
