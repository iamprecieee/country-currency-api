use crate::db::repositories::CountryRepository;

#[derive(Clone)]
pub struct AppState {
    pub repository: CountryRepository,
}
