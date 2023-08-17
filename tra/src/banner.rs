use crate::{info, replica::path_local::PathLocal};

pub struct LocalBanner;

pub struct SyncBanner;

impl LocalBanner {
    pub fn create(parent: &PathLocal, name: &String) {
        info!("Local create: \"{}\" in \"{}\"", name, parent.display())
    }

    pub fn modify(path: &PathLocal) {
        info!("Local modify: \"{}\"", path.display())
    }

    pub fn delete(path: &PathLocal) {
        info!("Local delete: \"{}\"", path.display())
    }
}

impl SyncBanner {
    pub fn skip_both_deleted(path: &PathLocal) {
        info!("Sync skip : \"{}\" (both deleted)", path.display())
    }

    pub fn skip_newer(path: &PathLocal) {
        info!("Sync skip : \"{}\" (newer)", path.display())
    }

    pub fn skip_from_independent_empty(path: &PathLocal) {
        info!(
            "Sync skip : \"{}\" (from independent empty)",
            path.display()
        )
    }

    pub fn skip_different_type(path: &PathLocal) {
        info!("Sync skip : \"{}\" (different type)", path.display())
    }

    pub fn delete(path: &PathLocal) {
        info!("Sync delete : \"{}\"", path.display())
    }

    pub fn create_to_independent_empty(path: &PathLocal) {
        info!(
            "Sync create : \"{}\" (to independent empty)",
            path.display()
        )
    }

    pub fn create_for_parent(path: &PathLocal) {
        info!("Sync create : \"{}\" (for parent)", path.display())
    }

    pub fn overwrite(path: &PathLocal) {
        info!("Sync overwrite : \"{}\"", path.display())
    }

    pub fn conflict(path: &PathLocal) {
        info!("Sync conflict : \"{}\"", path.display())
    }
}
