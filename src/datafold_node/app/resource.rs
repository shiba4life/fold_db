use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{FoldDbError, FoldDbResult};

/// Manages resource allocation for apps
#[derive(Clone)]
pub struct AppResourceManager {
    /// Map of app resource allocations
    allocations: HashMap<String, ResourceAllocation>,
    
    /// Map of app resource limits
    limits: HashMap<String, ResourceLimits>,
    
    /// System capacity
    system_capacity: SystemCapacity,
}

/// Represents resource allocation for an app
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    /// Memory allocation in bytes
    pub memory: usize,
    
    /// CPU allocation as percentage (0-100)
    pub cpu: f64,
    
    /// Storage allocation in bytes
    pub storage: usize,
    
    /// Bandwidth allocation in bytes per second
    pub bandwidth: usize,
}

/// Represents resource limits for an app
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory: usize,
    
    /// Maximum CPU as percentage (0-100)
    pub max_cpu: f64,
    
    /// Maximum storage in bytes
    pub max_storage: usize,
    
    /// Maximum bandwidth in bytes per second
    pub max_bandwidth: usize,
}

/// Represents system capacity
#[derive(Debug, Clone)]
struct SystemCapacity {
    /// Total memory in bytes
    total_memory: usize,
    
    /// Total CPU as percentage (0-100)
    total_cpu: f64,
    
    /// Total storage in bytes
    total_storage: usize,
    
    /// Total bandwidth in bytes per second
    total_bandwidth: usize,
    
    /// Used memory in bytes
    used_memory: usize,
    
    /// Used CPU as percentage (0-100)
    used_cpu: f64,
    
    /// Used storage in bytes
    used_storage: usize,
    
    /// Used bandwidth in bytes per second
    used_bandwidth: usize,
}

impl AppResourceManager {
    /// Creates a new resource manager
    pub fn new() -> Self {
        // Default system capacity
        let system_capacity = SystemCapacity {
            total_memory: 1024 * 1024 * 1024, // 1 GB
            total_cpu: 100.0,                 // 100%
            total_storage: 10 * 1024 * 1024 * 1024, // 10 GB
            total_bandwidth: 10 * 1024 * 1024, // 10 MB/s
            used_memory: 0,
            used_cpu: 0.0,
            used_storage: 0,
            used_bandwidth: 0,
        };
        
        Self {
            allocations: HashMap::new(),
            limits: HashMap::new(),
            system_capacity,
        }
    }
    
    /// Allocates resources for an app
    pub fn allocate_resources(&mut self, app_name: &str, resources: ResourceAllocation) -> FoldDbResult<()> {
        // Check if app already has resources allocated
        if self.allocations.contains_key(app_name) {
            return Err(FoldDbError::Config(format!("App '{}' already has resources allocated", app_name)));
        }
        
        // Check against system capacity
        if self.system_capacity.used_memory + resources.memory > self.system_capacity.total_memory {
            return Err(FoldDbError::Config("Not enough memory available".to_string()));
        }
        
        if self.system_capacity.used_cpu + resources.cpu > self.system_capacity.total_cpu {
            return Err(FoldDbError::Config("Not enough CPU available".to_string()));
        }
        
        if self.system_capacity.used_storage + resources.storage > self.system_capacity.total_storage {
            return Err(FoldDbError::Config("Not enough storage available".to_string()));
        }
        
        if self.system_capacity.used_bandwidth + resources.bandwidth > self.system_capacity.total_bandwidth {
            return Err(FoldDbError::Config("Not enough bandwidth available".to_string()));
        }
        
        // Check against app limits
        if let Some(limits) = self.limits.get(app_name) {
            if resources.memory > limits.max_memory {
                return Err(FoldDbError::Config("Memory allocation exceeds app limit".to_string()));
            }
            
            if resources.cpu > limits.max_cpu {
                return Err(FoldDbError::Config("CPU allocation exceeds app limit".to_string()));
            }
            
            if resources.storage > limits.max_storage {
                return Err(FoldDbError::Config("Storage allocation exceeds app limit".to_string()));
            }
            
            if resources.bandwidth > limits.max_bandwidth {
                return Err(FoldDbError::Config("Bandwidth allocation exceeds app limit".to_string()));
            }
        }
        
        // Update system capacity
        self.system_capacity.used_memory += resources.memory;
        self.system_capacity.used_cpu += resources.cpu;
        self.system_capacity.used_storage += resources.storage;
        self.system_capacity.used_bandwidth += resources.bandwidth;
        
        // Allocate resources
        self.allocations.insert(app_name.to_string(), resources);
        
        Ok(())
    }
    
    /// Releases resources for an app
    pub fn release_resources(&mut self, app_name: &str) -> FoldDbResult<()> {
        // Check if app has resources allocated
        let resources = self.allocations.remove(app_name)
            .ok_or_else(|| FoldDbError::Config(format!("App '{}' has no resources allocated", app_name)))?;
        
        // Update system capacity
        self.system_capacity.used_memory -= resources.memory;
        self.system_capacity.used_cpu -= resources.cpu;
        self.system_capacity.used_storage -= resources.storage;
        self.system_capacity.used_bandwidth -= resources.bandwidth;
        
        Ok(())
    }
    
    /// Sets resource limits for an app
    pub fn set_resource_limits(&mut self, app_name: &str, limits: ResourceLimits) -> FoldDbResult<()> {
        // Check if app already has resources allocated
        if let Some(allocation) = self.allocations.get(app_name) {
            // Check if new limits are below current allocation
            if allocation.memory > limits.max_memory {
                return Err(FoldDbError::Config("Memory limit is below current allocation".to_string()));
            }
            
            if allocation.cpu > limits.max_cpu {
                return Err(FoldDbError::Config("CPU limit is below current allocation".to_string()));
            }
            
            if allocation.storage > limits.max_storage {
                return Err(FoldDbError::Config("Storage limit is below current allocation".to_string()));
            }
            
            if allocation.bandwidth > limits.max_bandwidth {
                return Err(FoldDbError::Config("Bandwidth limit is below current allocation".to_string()));
            }
        }
        
        // Set limits
        self.limits.insert(app_name.to_string(), limits);
        
        Ok(())
    }
    
    /// Gets resource allocation for an app
    pub fn get_resource_allocation(&self, app_name: &str) -> Option<&ResourceAllocation> {
        self.allocations.get(app_name)
    }
    
    /// Gets resource limits for an app
    pub fn get_resource_limits(&self, app_name: &str) -> Option<&ResourceLimits> {
        self.limits.get(app_name)
    }
    
    /// Gets system capacity
    pub fn get_system_capacity(&self) -> SystemCapacityInfo {
        SystemCapacityInfo {
            total_memory: self.system_capacity.total_memory,
            total_cpu: self.system_capacity.total_cpu,
            total_storage: self.system_capacity.total_storage,
            total_bandwidth: self.system_capacity.total_bandwidth,
            used_memory: self.system_capacity.used_memory,
            used_cpu: self.system_capacity.used_cpu,
            used_storage: self.system_capacity.used_storage,
            used_bandwidth: self.system_capacity.used_bandwidth,
        }
    }
}

/// Represents system capacity information
#[derive(Debug, Clone)]
pub struct SystemCapacityInfo {
    /// Total memory in bytes
    pub total_memory: usize,
    
    /// Total CPU as percentage (0-100)
    pub total_cpu: f64,
    
    /// Total storage in bytes
    pub total_storage: usize,
    
    /// Total bandwidth in bytes per second
    pub total_bandwidth: usize,
    
    /// Used memory in bytes
    pub used_memory: usize,
    
    /// Used CPU as percentage (0-100)
    pub used_cpu: f64,
    
    /// Used storage in bytes
    pub used_storage: usize,
    
    /// Used bandwidth in bytes per second
    pub used_bandwidth: usize,
}
