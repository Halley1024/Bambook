use std::sync::Arc;

use crate::{
    application::ports::SettingsRepository, domain::settings::AppSettings, error::AppResult,
};

pub(crate) struct SettingsService {
    repository: Arc<dyn SettingsRepository>,
}

impl SettingsService {
    pub(crate) fn new(repository: Arc<dyn SettingsRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn load(&self) -> AppResult<AppSettings> {
        self.repository.load()
    }

    pub(crate) fn save(&self, settings: &AppSettings) -> AppResult<()> {
        self.repository.save(settings)
    }
}
