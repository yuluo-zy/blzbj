use utils::BResult;

pub struct Settings {

}

impl Settings {
    pub fn init() -> Self {
        Self {}
    }
}

pub struct SettingsManager {
    settings: Settings
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self {
            settings: Settings::init()
        }
    }
}
impl SettingsManager {
    pub fn get_setting(&self, key: &str) -> BResult<String> {
        Ok("".to_string())
    }
}