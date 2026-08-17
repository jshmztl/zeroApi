//! Settings 服务

use crate::domain::Settings;
use crate::repository::Repos;
use crate::AppResult;

#[derive(Clone)]
pub struct SettingsService {
    repos: Repos,
}

impl SettingsService {
    pub fn new(repos: &Repos) -> Self {
        Self {
            repos: repos.clone(),
        }
    }

    pub fn get(&self) -> AppResult<Settings> {
        self.repos.settings.get()
    }

    pub fn save(&self, settings: &Settings) -> AppResult<()> {
        self.repos.settings.save(settings)
    }
}
