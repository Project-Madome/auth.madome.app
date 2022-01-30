mod inmemory;
pub mod r#trait;

pub use inmemory::*;

use std::sync::Arc;

use sai::{Component, Injected};

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct RepositorySet {
    #[injected]
    authcode_repository: Injected<InMemoryAuthcodeRepository>,

    #[injected]
    secret_key_repository: Injected<InMemorySecretKeyRepository>,
}

impl RepositorySet {
    pub fn authcode(&self) -> Arc<impl r#trait::AuthcodeRepository> {
        Arc::clone(&self.authcode_repository)
    }

    pub fn secret_key(&self) -> Arc<impl r#trait::SecretKeyRepository> {
        Arc::clone(&self.secret_key_repository)
    }
}

#[cfg(test)]
mod tests {
    /* use sai::Injected;

    use super::InMemorySecretKeyRepository;

    impl super::RepositorySet {
        pub fn set_secret_key(&mut self, r: Injected<InMemorySecretKeyRepository>) {
            self.secret_key_repository = r;
        }
    } */
}
