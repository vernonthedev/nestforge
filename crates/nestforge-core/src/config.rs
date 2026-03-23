use crate::Container;
use anyhow::Result;
use nestforge_config::{ConfigOptions, ConfigService};

pub fn register_config(container: &Container, options: ConfigOptions) -> Result<()> {
    let config = ConfigService::load_with_options(&options)?;
    container.register(config).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nestforge_config::ConfigOptions;

    #[test]
    fn test_register_config_loads() {
        std::env::set_var("APP_NAME", "TestApp");

        let container = Container::new();
        let options = ConfigOptions::new();

        let result = register_config(&container, options);
        assert!(result.is_ok());

        let resolved = container.resolve::<ConfigService>();
        assert!(resolved.is_ok());
        assert_eq!(
            resolved.unwrap().get("APP_NAME"),
            Some("TestApp".to_string())
        );

        std::env::remove_var("APP_NAME");
    }
}
