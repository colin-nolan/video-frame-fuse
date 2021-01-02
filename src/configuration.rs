use serde::{Deserialize, Serialize};

#[derive(Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct BlackAndWhiteConfiguration {
    pub(crate) threshold: u8,
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
        BlackAndWhiteConfiguration {
            threshold: u8::max_value() / 2,
        }
    }
}
