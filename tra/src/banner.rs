use std::path::Path;

use crate::replica::path_local::PathLocal;

pub struct BannerOut;

pub struct LocalBanner;

pub struct SyncBanner;

impl BannerOut {
    pub fn check(msg: impl AsRef<str>) {
        println!("✔  {}", msg.as_ref());
    }

    pub fn new_watch(msg: impl AsRef<str>) {
        println!("👀 {}", msg.as_ref());
    }

    pub fn remove_watch(msg: impl AsRef<str>) {
        println!("\x1b[9m👀\x1b[0m {}", msg.as_ref());
    }

    pub fn new_sync(msg: impl AsRef<str>) {
        println!("🔄 {}", msg.as_ref());
    }

    pub fn event(msg: impl AsRef<str>) {
        println!("📢 {}", msg.as_ref());
    }

    pub fn cross(msg: impl AsRef<str>) {
        println!("❌ {}", msg.as_ref());
    }

    pub fn resolve(msg: impl AsRef<str>) {
        println!("🔧 {}", msg.as_ref());
    }

    pub fn warn(msg: impl AsRef<str>) {
        println!("⚠️ {}", msg.as_ref());
    }
}

impl LocalBanner {
    pub fn new_watch(path: impl AsRef<Path>) {
        BannerOut::new_watch(format!(
            "Local Watch: \"{}\" added",
            path.as_ref().display()
        ));
    }

    pub fn remove_watch(path: impl AsRef<Path>) {
        BannerOut::remove_watch(format!(
            "Local Watch: \"{}\" removed",
            path.as_ref().display()
        ));
    }

    pub fn create(parent: &PathLocal, name: &String) {
        BannerOut::event(format!(
            "Local Creation: \"{}\" in \"{}\"",
            name,
            parent.display()
        ));
    }

    pub fn modify(path: &PathLocal) {
        BannerOut::event(format!("Local Modification: \"{}\"", path.display()));
    }

    pub fn delete(path: &PathLocal) {
        BannerOut::event(format!("Local Deletion: \"{}\"", path.display()));
    }
}

impl SyncBanner {
    pub fn sync_request(id1: i32, port1: u16, id2: i32, prot2: u16, path: impl AsRef<Path>) {
        BannerOut::new_sync(format!(
            "Sync Request : replica-{}({}) -> replica-{}({}), path = \"{}\"",
            id1,
            port1,
            id2,
            prot2,
            path.as_ref().display()
        ));
    }

    pub fn skip_both_deleted(path: &PathLocal) {
        BannerOut::check(format!("Sync Skip : \"{}\" (both deleted)", path.display()));
    }

    pub fn skip_newer(path: &PathLocal) {
        BannerOut::check(format!("Sync Skip : \"{}\" (newer)", path.display()));
    }

    pub fn skip_from_independent_empty(path: &PathLocal) {
        BannerOut::check(format!(
            "Sync Skip : \"{}\" (from independent empty)",
            path.display()
        ));
    }

    pub fn skip_different_type(path: &PathLocal) {
        BannerOut::check(format!(
            "Sync Skip : \"{}\" (different type)",
            path.display()
        ));
    }

    pub fn delete(path: &PathLocal) {
        BannerOut::check(format!("Sync Deletion : \"{}\"", path.display()));
    }

    pub fn create_to_independent_empty(path: &PathLocal) {
        BannerOut::check(format!(
            "Sync Creation : \"{}\" (to independent empty)",
            path.display()
        ));
    }

    pub fn create_for_parent(path: &PathLocal) {
        BannerOut::check(format!(
            "Sync Creation : \"{}\" (for parent)",
            path.display()
        ));
    }

    pub fn overwrite(path: &PathLocal) {
        BannerOut::check(format!("Sync Overwrite : \"{}\"", path.display()));
    }

    pub fn conflict(path: &PathLocal) {
        BannerOut::resolve(format!("Sync Conflict : \"{}\"", path.display()));
    }
}
