use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use skia_safe::RuntimeEffect;
use cosmic_text::{FontSystem, SwashCache};
use crate::AssetLoader;

/// Manages heavy shared resources (Fonts, Shaders, Asset Loading).
///
/// This struct is extracted from the `Director` to allow passing asset context
/// to the renderer and other systems without mutably borrowing the entire `Director`
/// (which would cause borrow checker conflicts with the Scene Graph).
#[derive(Clone)]
pub struct AssetManager {
    /// Asset loader for resolving file paths to bytes.
    pub loader: Arc<dyn AssetLoader>,
    /// Global shader cache.
    pub shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
    /// Shared Font System.
    pub font_system: Arc<Mutex<FontSystem>>,
    /// Shared Swash Cache.
    pub swash_cache: Arc<Mutex<SwashCache>>,
    /// Shared Typeface Cache.
    pub typeface_cache: Arc<Mutex<HashMap<cosmic_text::fontdb::ID, skia_safe::Typeface>>>,
}

impl AssetManager {
    /// Creates a new `AssetManager`.
    pub fn new(
        loader: Arc<dyn AssetLoader>,
        font_system: Arc<Mutex<FontSystem>>,
        swash_cache: Arc<Mutex<SwashCache>>,
        shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
        typeface_cache: Arc<Mutex<HashMap<cosmic_text::fontdb::ID, skia_safe::Typeface>>>,
    ) -> Self {
        Self {
            loader,
            shader_cache,
            font_system,
            swash_cache,
            typeface_cache,
        }
    }
}
