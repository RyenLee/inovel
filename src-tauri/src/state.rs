use std::sync::Mutex;
use tauri::AppHandle;

use crate::optimization::OptimizationEngine;
use crate::settings::SharedConfig;

pub struct AppState {
    pub app_handle: Mutex<Option<AppHandle>>,
    pub config: SharedConfig,
    pub optimization: OptimizationEngine,
}

impl AppState {
    pub fn new(config: SharedConfig, optimization: OptimizationEngine) -> Self {
        Self {
            app_handle: Mutex::new(None),
            config,
            optimization,
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut guard = self.app_handle.lock().unwrap();
        *guard = Some(handle);
    }

    pub fn get_app_handle(&self) -> Option<AppHandle> {
        self.app_handle.lock().unwrap().clone()
    }
}
