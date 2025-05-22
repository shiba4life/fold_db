use crate::schema::types::{Fold, errors::SchemaError};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// Manages Fold objects loaded from disk or memory.
///
/// Similar to [`SchemaCore`], `FoldManager` provides basic persistence
/// and retrieval operations for [`Fold`] definitions.
pub struct FoldManager {
    folds: Mutex<HashMap<String, Fold>>, // loaded folds
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
        let mut folds = self
            .folds
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        folds.insert(fold.name.clone(), fold);
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

    /// Unloads a fold from memory without deleting its file.
    pub fn unload_fold(&self, name: &str) -> Result<(), SchemaError> {
        let mut folds = self
            .folds
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        folds.remove(name);
        Ok(())
    }

    /// Lists all loaded folds.
    pub fn list_folds(&self) -> Result<Vec<String>, SchemaError> {
        let folds = self
            .folds
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire fold lock".to_string()))?;
        Ok(folds.keys().cloned().collect())
    }

    /// Loads all fold files from the folds directory.
    pub fn load_folds_from_disk(&self) -> Result<(), SchemaError> {
        if let Ok(entries) = fs::read_dir(&self.folds_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "json").unwrap_or(false)
                {
                    if let Ok(contents) = fs::read_to_string(entry.path()) {
                        if let Ok(fold) = serde_json::from_str::<Fold>(&contents) {
                            let _ = self.load_fold(fold);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Loads a fold from a JSON string.
    pub fn load_fold_from_json(&self, json: &str) -> Result<(), SchemaError> {
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
    }
}
