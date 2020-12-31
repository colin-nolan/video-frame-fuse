// TODO: move to appropriate module
// #[derive(Clone)]
pub struct FileInformation {
    pub name: String,
    pub data_fetcher: Option<Box<dyn Fn() -> Vec<u8>>>,
    pub data: Option<Vec<u8>>,
    pub initially_listed: bool,
    pub executable: bool,
}

impl FileInformation {
    pub fn new(
        name: &str,
        data_fetcher: Box<dyn Fn() -> Vec<u8>>,
        listed: bool,
        executable: bool,
    ) -> Self {
        FileInformation {
            name: name.to_string(),
            data_fetcher: Some(data_fetcher),
            data: None,
            initially_listed: listed,
            executable,
        }
    }

    pub fn new_with_data(name: &str, data: Vec<u8>, listed: bool, executable: bool) -> Self {
        FileInformation {
            name: name.to_string(),
            data_fetcher: None,
            data: Some(data),
            initially_listed: listed,
            executable,
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        match self.data_fetcher.is_some() {
            true => (self.data_fetcher.as_ref().unwrap())(),
            false => self.data.as_ref().unwrap().clone(),
        }
    }
}
