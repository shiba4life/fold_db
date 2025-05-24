use chrono::{DateTime, Utc};
use crate::atom::atom_ref_types::{AtomRefStatus, AtomRefUpdate};

/// A trait defining the common behavior for atom references.
///
/// This trait provides the interface for both single atom references
/// and collections of atom references.
pub trait AtomRefBehavior {
    /// Returns the unique identifier of this reference
    fn uuid(&self) -> &str;

    /// Returns the timestamp of the last update
    fn updated_at(&self) -> DateTime<Utc>;

    /// Returns the status of this reference
    fn status(&self) -> &AtomRefStatus;

    /// Sets the status of this reference
    fn set_status(&mut self, status: &AtomRefStatus, source_pub_key: String);

    /// Returns the update history
    fn update_history(&self) -> &Vec<AtomRefUpdate>;
}