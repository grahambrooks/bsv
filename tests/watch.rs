//! End-to-end test of the filesystem watcher: a real edit to a catalog file
//! under the watched directory should produce a change signal.

use bsv::watcher::CatalogWatcher;
use std::fs;
use std::time::{Duration, Instant};

#[test]
fn detects_catalog_file_change() {
    // Unique temp directory (avoids Date/rand; uses pid + a nonce file).
    let mut dir = std::env::temp_dir();
    dir.push(format!("bsv-watch-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let file = dir.join("catalog-info.yaml");
    fs::write(
        &file,
        "apiVersion: backstage.io/v1alpha1\nkind: Component\n",
    )
    .unwrap();

    let watcher = CatalogWatcher::new(&dir).expect("start watcher");

    // Give the watcher a moment to initialize, then modify the file.
    std::thread::sleep(Duration::from_millis(200));
    fs::write(
        &file,
        "apiVersion: backstage.io/v1alpha1\nkind: Component\nmetadata:\n  name: changed\n",
    )
    .unwrap();

    // Poll for the change signal with a generous timeout.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut seen = false;
    while Instant::now() < deadline {
        if watcher.drain() {
            seen = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let _ = fs::remove_dir_all(&dir);
    assert!(seen, "watcher should observe the catalog file change");
}
