use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum ConfigurationHolder {
    None,
    BlackAndWhite(BlackAndWhiteConfiguration),
}

pub trait Configuration {
    fn from_yaml(yaml_string: &str) -> Result<Self, String>
    where
        Self: Sized;

    fn to_yaml(&self) -> Result<String, String>
    where
        Self: serde::Serialize,
    {
        match serde_yaml::to_string(self) {
            Ok(x) => Ok(x),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlackAndWhiteConfiguration {
    pub(crate) threshold: Option<u8>,
}

impl Configuration for BlackAndWhiteConfiguration {
    fn from_yaml(yaml_string: &str) -> Result<Self, String> {
        match serde_yaml::from_str::<BlackAndWhiteConfiguration>(&yaml_string) {
            Ok(x) => Ok(x),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Default for BlackAndWhiteConfiguration {
    fn default() -> Self {
        BlackAndWhiteConfiguration { threshold: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(None; "when threshold is none")]
    #[test_case(Some(0); "when threshold is 0")]
    #[test_case(Some(255); "when threshold is 255")]
    fn black_and_white_configuration_serialisation(threshold: Option<u8>) {
        let configuration = BlackAndWhiteConfiguration { threshold };
        let yaml = configuration.to_yaml().unwrap();
        let parsed_configuration = BlackAndWhiteConfiguration::from_yaml(&yaml).unwrap();
        assert_eq!(parsed_configuration, configuration);
    }
}
