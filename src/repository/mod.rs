pub mod authcode_repository;

use std::sync::Arc;

pub use authcode_repository::InMemoryAuthcodeRepository;
use sai::{Component, ComponentLifecycle, Injected};

pub mod r#trait {
    pub use super::authcode_repository::r#trait::AuthcodeRepository;
}

#[derive(Component)]
#[lifecycle]
pub struct RepositorySet {
    #[injected]
    authcode_repository: Injected<InMemoryAuthcodeRepository>,
}

impl RepositorySet {
    pub fn authcode(&self) -> Arc<impl r#trait::AuthcodeRepository> {
        Arc::clone(&self.authcode_repository)
    }
}

#[async_trait::async_trait]
impl ComponentLifecycle for RepositorySet {
    async fn start(&mut self) {}
}
