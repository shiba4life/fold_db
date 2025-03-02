mod manifest;
mod registry;
mod loader;
mod window;
pub mod api;
mod resource;

pub use manifest::{AppManifest, WindowConfig, AppPermissions, ApiRequirements, PaymentConfig};
pub use registry::AppRegistry;
pub use loader::AppLoader;
pub use window::AppWindow;
pub use api::ApiManager;
pub use resource::{AppResourceManager, ResourceAllocation, ResourceLimits};

use crate::error::FoldDbResult;

/// Initializes the app system
pub fn init_app_system() -> FoldDbResult<()> {
    // Initialize app registry
    let _registry = AppRegistry::new();
    
    // Initialize resource manager
    let _resource_manager = AppResourceManager::new();
    
    // Initialize API manager
    let _api_manager = ApiManager::new();
    
    Ok(())
}
