//! Filesystem watching for automatic catalog reloads.
//!
//! [`CatalogWatcher`] watches the catalog path and signals when a relevant YAML
//! file changes. The event loop drains it and reloads (with debouncing) so the
//! UI reflects on-disk edits without a manual reload.

use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};

/// Watches a catalog directory (or a file's directory) for changes.
pub struct CatalogWatcher {
    // Held to keep the watch alive; dropping it stops watching.
    _watcher: RecommendedWatcher,
    rx: Receiver<()>,
}

impl CatalogWatcher {
    /// Start watching `root`. If `root` is a file, its parent directory is
    /// watched (filesystem watchers operate on directories).
    pub fn new(root: &Path) -> Result<Self> {
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if is_relevant(&event) {
                    // Ignore send errors: the receiver may have been dropped.
                    let _ = tx.send(());
                }
            }
        })?;

        let watch_path = if root.is_file() {
            root.parent().unwrap_or(root)
        } else {
            root
        };
        watcher.watch(watch_path, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Drain pending events, returning `true` if any relevant change occurred
    /// since the last call.
    pub fn drain(&self) -> bool {
        let mut changed = false;
        // Stops on the first Empty/Disconnected error.
        while let Ok(()) = self.rx.try_recv() {
            changed = true;
        }
        changed
    }
}

/// Whether a filesystem event should trigger a reload: a create/modify/remove
/// touching a YAML file.
fn is_relevant(event: &Event) -> bool {
    let interesting_kind = matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    );
    interesting_kind
        && event.paths.iter().any(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("yaml") || e.eq_ignore_ascii_case("yml"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{AccessKind, CreateKind, ModifyKind};
    use std::path::PathBuf;

    fn event(kind: EventKind, path: &str) -> Event {
        Event {
            kind,
            paths: vec![PathBuf::from(path)],
            attrs: Default::default(),
        }
    }

    #[test]
    fn yaml_changes_are_relevant() {
        assert!(is_relevant(&event(
            EventKind::Create(CreateKind::File),
            "/x/catalog-info.yaml"
        )));
        assert!(is_relevant(&event(
            EventKind::Modify(ModifyKind::Any),
            "/x/catalog-info.yml"
        )));
    }

    #[test]
    fn non_yaml_changes_are_ignored() {
        assert!(!is_relevant(&event(
            EventKind::Create(CreateKind::File),
            "/x/README.md"
        )));
        assert!(!is_relevant(&event(
            EventKind::Access(AccessKind::Any),
            "/x/catalog-info.yaml"
        )));
    }

    #[test]
    fn watcher_starts_on_directory() {
        let watcher = CatalogWatcher::new(Path::new("testdata")).expect("watch testdata");
        // No events yet.
        assert!(!watcher.drain());
    }
}
