use pkgseal_source::registry::SourceRegistry;
use pkgseal_source_arch::ArchSource;
use pkgseal_source_aur::AurSource;
use pkgseal_source_flatpak::FlatpakSource;
use std::sync::Arc;

pub struct AppState {
    pub registry: SourceRegistry,
}

impl AppState {
    pub fn new() -> Self {
        let mut registry = SourceRegistry::new();
        registry.register(Arc::new(ArchSource::new()));
        registry.register(Arc::new(AurSource::new()));
        registry.register(Arc::new(FlatpakSource::new()));
        Self { registry }
    }
}
