use std::sync::RwLock;

static CONFIG: RwLock<DebugConfig> = RwLock::new(default_config());

/// Configures debug-only signal runtime checks.
///
/// In release builds this function has no effect.
pub fn debug_signals(config: DebugConfig) {
    #[cfg(debug_assertions)]
    {
        let mut write = CONFIG.write().unwrap();
        *write = config;
    }
}

pub(crate) fn get_config() -> DebugConfig {
    let read = CONFIG.read().unwrap();
    *read
}

/// Debug-only runtime options for the signal system.
#[derive(Debug, Clone, Copy)]
pub struct DebugConfig {
    /// Warns when an effect attempts to trigger itself while it is already
    /// running.
    pub detect_cyclic: bool,
}

const fn default_config() -> DebugConfig {
    DebugConfig {
        detect_cyclic: false,
    }
}
