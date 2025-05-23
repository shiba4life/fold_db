use crate::schema::types::{Fold, errors::SchemaError};
use crate::schema::types::JsonFoldDefinition;
use std::convert::TryFrom;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// State of a fold within the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldState {
    Loaded,
    Unloaded,
}

/// Manages Fold objects loaded from disk or memory.
///
/// Similar to [`SchemaCore`], `FoldManager` provides basic persistence
/// and retrieval operations for [`Fold`] definitions.
pub struct FoldManager {
    folds: Mutex<HashMap<String, Fold>>, // loaded folds
    /// All folds known to the system and their load state
    available: Mutex<HashMap<String, (Fold, FoldState)>>,
    folds_dir: PathBuf,
}

impl FoldManager {
    fn init_with_dir(folds_dir: PathBuf) -> Result<Self, SchemaError> {
        if let Err(e) = fs::create_dir_all(&folds_dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(SchemaError::InvalidData(format!(
                    "Failed to create folds directory: {}",
                    e
                )));
            }
        }
        Ok(Self {
            folds: Mutex::new(HashMap::new()),
            available: Mutex::new(HashMap::new()),
            folds_dir,
        })
    }

    /// Creates a new manager using the default `data/folds` directory.
    pub fn init_default() -> Result<Self, SchemaError> {
        Self::init_with_dir(PathBuf::from("data/folds"))
    }

    /// Creates a new manager with a custom base path.
    pub fn new(path: &str) -> Result<Self, SchemaError> {
        let folds_dir = PathBuf::from(path).join("folds");
        Self::init_with_dir(folds_dir)
    }

    fn fold_path(&self, name: &str) -> PathBuf {
        self.folds_dir.join(format!("{}.json", name))
    }

    fn persist_fold(&self, fold: &Fold) -> Result<(), SchemaError> {
        let path = self.fold_path(&fold.name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to create fold directory: {}", e))
            })?;
        }
        let json = serde_json::to_string_pretty(fold)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize fold: {}", e)))?;
        fs::write(&path, json).map_err(|e| {
            SchemaError::InvalidData(format!(
                "Failed to write fold file: {}, path: {}",
                e,
                path.to_string_lossy()
            ))
        })?;
        Ok(())
    }

    /// Loads a fold into memory and persists it to disk.
    pub fn load_fold(&self, fold: Fold) -> Result<(), SchemaError> {
        self.persist_fold(&fold)?;
        {
            let mut folds = self
                .folds
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
            folds.insert(fold.name.clone(), fold.clone());
        }
        let mut all = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        all.insert(fold.name.clone(), (fold, FoldState::Loaded));
        Ok(())
    }

    /// Retrieves a fold by name.
    pub fn get_fold(&self, name: &str) -> Result<Option<Fold>, SchemaError> {
        let folds = self
            .folds
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        Ok(folds.get(name).cloned())
    }

    /// Mark a fold as unloaded but keep it available in memory
    pub fn set_unloaded(&self, name: &str) -> Result<(), SchemaError> {
        let mut folds = self
            .folds
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        folds.remove(name);
        let mut available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        if let Some((_, state)) = available.get_mut(name) {
            *state = FoldState::Unloaded;
            Ok(())
        } else {
            Err(SchemaError::NotFound(format!("Fold {name} not found")))
        }
    }

    /// Unloads a fold from memory without deleting its file.
    pub fn unload_fold(&self, name: &str) -> Result<(), SchemaError> {
        self.set_unloaded(name)
    }

    /// Lists all loaded folds.
    pub fn list_loaded_folds(&self) -> Result<Vec<String>, SchemaError> {
        let folds = self
            .folds
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        Ok(folds.keys().cloned().collect())
    }

    /// Lists all folds available on disk and their state.
    pub fn list_available_folds(&self) -> Result<Vec<String>, SchemaError> {
        let available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        Ok(available.keys().cloned().collect())
    }

    /// Backwards compatible method for listing loaded folds.
    pub fn list_folds(&self) -> Result<Vec<String>, SchemaError> {
        self.list_loaded_folds()
    }

    /// Checks if a fold exists in the manager.
    pub fn fold_exists(&self, fold_name: &str) -> Result<bool, SchemaError> {
        let folds = self
            .folds
            .lock()
            .map_err(|_| {
                SchemaError::InvalidData("Failed to acquire fold lock".to_string())
            })?;
        Ok(folds.contains_key(fold_name))
    }

    /// Loads all fold files from the folds directory.
    pub fn load_folds_from_disk(&self) -> Result<(), SchemaError> {
        let mut errors = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.folds_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "json").unwrap_or(false)
                {
                    match fs::read_to_string(entry.path()) {
                        Ok(contents) => {
                            match serde_json::from_str::<Fold>(&contents) {
                                Ok(fold) => {
                                    if let Err(e) = self.load_fold(fold) {
                                        errors.push(e.to_string());
                                    }
                                }
                                Err(_) => match serde_json::from_str::<JsonFoldDefinition>(&contents) {
                                    Ok(json_fold) => match Fold::try_from(json_fold) {
                                        Ok(fold) => {
                                            if let Err(e) = self.load_fold(fold) {
                                                errors.push(e.to_string());
                                            }
                                        }
                                        Err(e) => errors.push(e.to_string()),
                                    },
                                    Err(e) => errors.push(e.to_string()),
                                },
                            }
                        }
                        Err(e) => errors.push(e.to_string()),
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(SchemaError::InvalidData(format!(
                "Failed to load folds: {}",
                errors.join("; ")
            )))
        }
    }

    /// Loads a fold from a JSON string.
    pub fn load_fold_from_json(&self, json: &str) -> Result<(), SchemaError> {
        if let Ok(json_fold) = serde_json::from_str::<JsonFoldDefinition>(json) {
            let fold = Fold::try_from(json_fold)?;
            return self.load_fold(fold);
        }
        let fold: Fold = serde_json::from_str(json)
            .map_err(|e| SchemaError::InvalidData(format!("Invalid JSON fold: {}", e)))?;
        self.load_fold(fold)
    }

    /// Loads a fold from a file path.
    pub fn load_fold_from_file(&self, path: &str) -> Result<(), SchemaError> {
        let json = fs::read_to_string(path)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read fold file: {e}")))?;
        self.load_fold_from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::types::policy::PermissionsPolicy;
    use crate::schema::types::field::{Field, FieldVariant, SingleField};
    use crate::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
    use tempfile::tempdir;
    use uuid::Uuid;

    fn create_fold(name: &str) -> Fold {
        let mut fold = Fold::new(name.to_string());
        let mut field = SingleField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
            HashMap::new(),
        );
        field.set_ref_atom_uuid(Uuid::new_v4().to_string());
        fold.add_field("field".to_string(), FieldVariant::Single(field));
        fold
    }

    #[test]
    fn test_load_and_get_fold() {
        let dir = tempdir().unwrap();
        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        let fold = create_fold("sample");
        manager.load_fold(fold.clone()).unwrap();
        let loaded = manager.get_fold("sample").unwrap().unwrap();
        assert_eq!(loaded.name, fold.name);
    }

    #[test]
    fn test_persist_and_reload_fold() {
        let dir = tempdir().unwrap();
        {
            let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
            let fold = create_fold("persist");
            manager.load_fold(fold).unwrap();
        }
        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        manager.load_folds_from_disk().unwrap();
        assert!(manager.get_fold("persist").unwrap().is_some());
    }

    #[test]
    fn test_unload_fold() {
        let dir = tempdir().unwrap();
        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        let fold = create_fold("temp");
        manager.load_fold(fold).unwrap();
        manager.unload_fold("temp").unwrap();
        assert!(manager.get_fold("temp").unwrap().is_none());
        let available = manager.list_available_folds().unwrap();
        assert!(available.contains(&"temp".to_string()));
    }

    #[test]
    fn test_fold_exists() {
        let dir = tempdir().unwrap();
        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        let fold = create_fold("exists");
        manager.load_fold(fold).unwrap();
        assert!(manager.fold_exists("exists").unwrap());
        assert!(!manager.fold_exists("missing").unwrap());
        manager.unload_fold("exists").unwrap();
        assert!(!manager.fold_exists("exists").unwrap());
    }

    #[test]
    fn test_list_loaded_and_available() {
        let dir = tempdir().unwrap();
        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        manager.load_fold(create_fold("a")).unwrap();
        manager.load_fold(create_fold("b")).unwrap();
        manager.unload_fold("b").unwrap();
        let loaded = manager.list_loaded_folds().unwrap();
        assert_eq!(loaded, vec!["a".to_string()]);
        let available = manager.list_available_folds().unwrap();
        assert!(available.contains(&"a".to_string()));
        assert!(available.contains(&"b".to_string()));
    }

    #[test]
    fn test_load_fold_from_json_definition() {
        let dir = tempdir().unwrap();
        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        let json = r#"{
            "name": "json_fold",
            "fields": {
                "field": {
                    "permission_policy": {
                        "read_policy": { "Distance": 0 },
                        "write_policy": { "Distance": 0 },
                        "explicit_read_policy": null,
                        "explicit_write_policy": null
                    },
                    "ref_atom_uuid": "uuid1",
                    "payment_config": {
                        "base_multiplier": 1.0,
                        "trust_distance_scaling": { "None": null },
                        "min_payment": null
                    },
                    "field_mappers": {},
                    "field_type": "Single"
                }
            },
            "payment_config": {
                "base_multiplier": 1.0,
                "min_payment_threshold": 0
            }
        }"#;

        manager.load_fold_from_json(json).unwrap();
        assert!(manager.get_fold("json_fold").unwrap().is_some());
    }

    #[test]
    fn test_load_invalid_json_definition() {
        let dir = tempdir().unwrap();
        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        let json = r#"{
            "name": "bad_fold",
            "fields": {},
            "payment_config": { "base_multiplier": 0.0, "min_payment_threshold": 0 }
        }"#;
        let res = manager.load_fold_from_json(json);
        assert!(res.is_err());
    }

    #[test]
    fn test_load_folds_from_disk_invalid_file() {
        use std::fs::{self, File};
        use std::io::Write;

        let dir = tempdir().unwrap();
        let fold_dir = dir.path().join("folds");
        fs::create_dir_all(&fold_dir).unwrap();
        let file_path = fold_dir.join("bad.json");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"not json").unwrap();

        let manager = FoldManager::new(dir.path().to_str().unwrap()).unwrap();
        let res = manager.load_folds_from_disk();
        assert!(res.is_err());
    }
}
