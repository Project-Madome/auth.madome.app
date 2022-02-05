pub use self::root_registry::RootRegistry;

#[cfg(test)]
mod test_root_registry {
    use sai::{combine_component_registry, component_registry, Component};

    use crate::{
        command::{random_code::RandomCode, tests::GetUserInfo, tests::SendEmail, CommandSet},
        config::Config,
        repository::{InMemoryAuthcodeRepository, RepositorySet},
    };

    combine_component_registry!(RootRegistry, [RepositoryRegistry]);

    component_registry!(
        RepositoryRegistry,
        [RepositorySet, InMemoryAuthcodeRepository]
    );

    component_registry!(
        CommandRegistry,
        [CommandSet, GetUserInfo, RandomCode, SendEmail]
    );

    component_registry!(ConfigRegistry, [Config]);
}

mod root_registry {
    use sai::{combine_component_registry, component_registry, Component};

    use crate::{
        app::{HttpServer, Resolver},
        command::{random_code::RandomCode, send_email::SendEmail, CommandSet, GetUserInfo},
        config::Config,
        database::DatabaseSet,
        repository::{InMemoryAuthcodeRepository, RedisSecretKeyRepository, RepositorySet},
    };

    combine_component_registry!(
        RootRegistry,
        [
            ServerRegistry,
            ControllerRegistry,
            RepositoryRegistry,
            CommandRegistry,
            ConfigRegistry
        ]
    );

    component_registry!(ServerRegistry, [HttpServer]);

    component_registry!(ControllerRegistry, [Resolver]);

    component_registry!(
        RepositoryRegistry,
        [
            DatabaseSet,
            RepositorySet,
            InMemoryAuthcodeRepository,
            RedisSecretKeyRepository
        ]
    );

    component_registry!(
        CommandRegistry,
        [CommandSet, GetUserInfo, RandomCode, SendEmail]
    );

    component_registry!(ConfigRegistry, [Config]);
}
