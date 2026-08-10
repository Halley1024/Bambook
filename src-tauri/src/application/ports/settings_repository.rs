use crate::{domain::settings::AppSettings, error::AppResult};

pub(crate) trait SettingsRepository: Send + Sync {
    fn load(&self) -> AppResult<AppSettings>;
    fn save(&self, settings: &AppSettings) -> AppResult<()>;
}
