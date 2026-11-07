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

    /// Queue a file for the frame loop. A file already loaded, in flight or
    /// failed is never queued twice.
    pub(crate) fn want(path: &'static str) {
        if loaded(path).is_some()
            || IN_FLIGHT.with(Cell::get) == Some(path)
            || FAILED.with(|failed| failed.borrow().contains(path))
        {
            return;
        }
        QUEUE.with(|queue| {
            let mut queue = queue.borrow_mut();
            if !queue.contains(&path) {
                queue.push_back(path);
            }
        });
    }

    /// True while artwork is queued or arriving; the frame loop draws the
    /// loading screen instead of the game.
    pub(crate) fn pending() -> bool {
        QUEUE.with(|queue| !queue.borrow().is_empty()) || IN_FLIGHT.with(Cell::get).is_some()
    }

    /// Install a finished download and start the next one. One call per frame:
    /// miniquad's callback API keeps the frame loop free to animate.
    pub(crate) fn pump() {
        if let Some((path, bytes)) = ARRIVED.with(|slot| slot.borrow_mut().take()) {
            IN_FLIGHT.with(|slot| slot.set(None));
            match bytes {
                Some(bytes) => {
                    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                    LOADED.with(|loaded| loaded.borrow_mut().insert(path, bytes));
                }
                None => {
                    FAILED.with(|failed| failed.borrow_mut().insert(path));
                }
            }
        }
        if IN_FLIGHT.with(Cell::get).is_some() {
            return;
        }
        let Some(path) = QUEUE.with(|queue| queue.borrow_mut().pop_front()) else {
            return;
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
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) use web::{pending, pump, want};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn want(_path: &'static str) {}

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn pump() {}

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
pub(crate) fn pending() -> bool {
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
