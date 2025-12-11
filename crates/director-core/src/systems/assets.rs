use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use skia_safe::RuntimeEffect;
use skia_safe::textlayout::{FontCollection, TypefaceFontProvider};
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
    /// Shared Font Collection (Skia).
    pub font_collection: Arc<Mutex<FontCollection>>,
    /// Shared Font Provider (Skia).
    pub font_provider: Arc<Mutex<TypefaceFontProvider>>,
}

impl AssetManager {
    /// Creates a new `AssetManager`.
    pub fn new(
        loader: Arc<dyn AssetLoader>,
        font_collection: Arc<Mutex<FontCollection>>,
        font_provider: Arc<Mutex<TypefaceFontProvider>>,
        shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
    ) -> Self {
        Self {
            loader,
            shader_cache,
            font_collection,
            font_provider,
        }
    }
}
