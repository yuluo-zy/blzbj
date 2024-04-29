use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use utils::BResult;
use utils::parking_lot::lock_api::MutexGuard;
use utils::parking_lot::{Mutex, RawMutex};
use crate::settings::SettingsManager;
use crate::task::task::TaskTait;

pub struct Manager {
    task_pool: HashMap<String, Box<dyn TaskTait>>,
    settings_manager: Arc<Mutex<SettingsManager>>, // 会被多线程中共享使用
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            task_pool: HashMap::new(),
            settings_manager: Arc::new(Mutex::new(SettingsManager::default()))
        }
    }
}
impl Manager {

    pub fn load_all_tasks(&self) -> BResult<bool> {
        let res = self.settings_manager.lock();
         for i in res.get_setting(&"task") {

         }
        Ok(true)
    }
}
