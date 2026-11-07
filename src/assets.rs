//! Runtime artwork bytes.
//!
//! Native builds embed every file with `include_bytes!`, as before. The web
//! build keeps only code, config and the font in the `.wasm` and fetches the
//! sprites and effect sheets from `assets/` beside it. The frame loop resolves
//! the requests the session makes, so the menu needs one hero atlas instead of
//! the whole roster.

/// One runtime artwork file.
///
/// The path is relative to `assets/` and is also the web fetch path.
#[derive(Clone, Copy)]
pub struct Asset {
    pub path: &'static str,
    #[cfg(not(target_arch = "wasm32"))]
    embedded: &'static [u8],
}

impl Asset {
    #[cfg(not(target_arch = "wasm32"))]
    pub const fn new(path: &'static str, embedded: &'static [u8]) -> Self {
        Self { path, embedded }
    }

    #[cfg(target_arch = "wasm32")]
    pub const fn new(path: &'static str) -> Self {
        Self { path }
    }

    /// Whether the bytes are in memory yet. Always true when they are embedded.
    pub fn available(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            true
        }
        #[cfg(target_arch = "wasm32")]
        {
            web::loaded(self.path).is_some()
        }
    }

    /// The encoded artwork. Ask for [`available`](Self::available) first on the
    /// web; drawing code falls back to a blank texture instead of calling this.
    pub fn bytes(&self) -> &'static [u8] {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.embedded
        }
        #[cfg(target_arch = "wasm32")]
        {
            web::loaded(self.path).unwrap_or_else(|| panic!("asset not loaded: {}", self.path))
        }
    }
}

/// Embedded on native; a web fetch request on wasm.
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! asset {
    ($path:literal) => {
        $crate::assets::Asset::new(
            $path,
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path)),
        )
    };
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! asset {
    ($path:literal) => {
        $crate::assets::Asset::new($path)
    };
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::{Cell, RefCell};
    use std::collections::{HashMap, HashSet, VecDeque};

    thread_local! {
        static LOADED: RefCell<HashMap<&'static str, &'static [u8]>> = RefCell::new(HashMap::new());
        static QUEUE: RefCell<VecDeque<&'static str>> = RefCell::new(VecDeque::new());
        static FAILED: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
        static IN_FLIGHT: Cell<Option<&'static str>> = const { Cell::new(None) };
        static ARRIVED: RefCell<Option<(&'static str, Option<Vec<u8>>)>> = RefCell::new(None);
    }

    pub(super) fn loaded(path: &'static str) -> Option<&'static [u8]> {
        LOADED.with(|loaded| loaded.borrow().get(path).copied())
    }

    /// Queue background artwork (the roster next to the selected hero).
    pub(crate) fn want(path: &'static str) {
        enqueue(path, false);
    }

    /// Queue artwork the current frame draws; jumps ahead of background work.
    pub(crate) fn require(path: &'static str) {
        enqueue(path, true);
    }

    fn enqueue(path: &'static str, front: bool) {
        if loaded(path).is_some()
            || IN_FLIGHT.with(Cell::get) == Some(path)
            || FAILED.with(|failed| failed.borrow().contains(path))
        {
            return;
        }
        QUEUE.with(|queue| {
            let mut queue = queue.borrow_mut();
            if queue.contains(&path) {
                if front {
                    queue.retain(|queued| *queued != path);
                    queue.push_front(path);
                }
                return;
            }
            if front {
                queue.push_front(path);
            } else {
                queue.push_back(path);
            }
        });
    }

    /// Install a finished download and start the next one. Returns whether any
    /// texture can be rebuilt this frame.
    pub(crate) fn pump() -> bool {
        let mut installed = false;
        if let Some((path, bytes)) = ARRIVED.with(|slot| slot.borrow_mut().take()) {
            IN_FLIGHT.with(|slot| slot.set(None));
            match bytes {
                Some(bytes) => {
                    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                    LOADED.with(|loaded| loaded.borrow_mut().insert(path, bytes));
                    installed = true;
                }
                None => {
                    FAILED.with(|failed| failed.borrow_mut().insert(path));
                }
            }
        }
        if IN_FLIGHT.with(Cell::get).is_some() {
            return installed;
        }
        let Some(path) = QUEUE.with(|queue| queue.borrow_mut().pop_front()) else {
            return installed;
        };
        IN_FLIGHT.with(|slot| slot.set(Some(path)));
        let url = format!("assets/{path}");
        let shown = url.clone();
        macroquad::miniquad::fs::load_file(&url, move |result| {
            let bytes = match result {
                Ok(bytes) => Some(bytes),
                Err(error) => {
                    eprintln!("asset {shown}: {error}");
                    None
                }
            };
            ARRIVED.with(|slot| *slot.borrow_mut() = Some((path, bytes)));
        });
        installed
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) use web::{pump, require, want};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn want(_path: &'static str) {}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn require(_path: &'static str) {}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(crate) fn pump() -> bool {
    false
}

/// Every artwork file the build may need at runtime.
///
/// Packaging copies exactly this inventory; the unit test checks it on disk.
/// Runtime loading is on demand, so no other code calls it.
#[allow(dead_code)]
pub fn required() -> Vec<&'static str> {
    let mut paths: Vec<&'static str> = crate::sprites::SPRITES
        .iter()
        .chain(crate::sprites::AWAKENED.iter())
        .map(|sprite| sprite.asset.path)
        .collect();
    paths.extend_from_slice(crate::effects::SHEET_ASSETS);
    paths.sort_unstable();
    paths.dedup();
    paths
}

#[cfg(test)]
mod tests {
    /// Packaging and the web loader both rely on the registry paths existing.
    #[test]
    fn every_runtime_asset_exists_on_disk() {
        for path in super::required() {
            let full = format!("{}/assets/{path}", env!("CARGO_MANIFEST_DIR"));
            assert!(
                std::path::Path::new(&full).is_file(),
                "missing runtime asset {path}"
            );
        }
    }
}
