use std::sync::RwLock;

static CONFIG: RwLock<DebugConfig> = RwLock::new(default_config());

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

#[derive(Debug, Clone, Copy)]
pub struct DebugConfig {
    pub detect_cyclic: bool,
}

const fn default_config() -> DebugConfig {
    DebugConfig {
        detect_cyclic: false,
    }
}
