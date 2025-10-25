use crate::{db::repositories::CountryRepository, utils::config::Config};

#[derive(Clone)]
pub struct AppState {
    pub repository: CountryRepository,
    pub config: Config,
}
