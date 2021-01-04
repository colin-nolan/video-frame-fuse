pub struct FileInformation {
    pub name: String,
    pub data_fetcher: Option<Box<dyn Fn() -> Vec<u8>>>,
    pub data: Option<Vec<u8>>,
    pub listed: bool,
    pub executable: bool,
    pub writable: bool,
    on_data_change: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
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
            data_fetcher: Some(Box::new(data_fetcher)),
            data: None,
            listed,
            executable,
            writable: false,
            on_data_change: None,
        }
    }

    pub fn new_with_data(
        name: &str,
        data: Vec<u8>,
        listed: bool,
        executable: bool,
        writable: bool,
        // TODO: does this need to be boxed?
        on_data_change: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    ) -> Self {
        FileInformation {
            name: name.to_string(),
            data_fetcher: None,
            data: Some(data),
            listed,
            executable,
            writable,
            on_data_change,
        }
    }

    pub fn set_data(&mut self, data: Vec<u8>) -> Result<(), String> {
        let parsed = match String::from_utf8(data.clone()) {
            Ok(x) => x,
            Err(e) => return Err(e.to_string()),
        };
        if self.on_data_change.is_some() {
            (self.on_data_change.as_ref().unwrap())(&parsed)?;
        }
        self.data = Some(data);
        Ok(())
    }

    pub fn get_data(&self) -> Vec<u8> {
        match &self.data_fetcher {
            None => self.data.as_ref().unwrap().clone(),
            Some(x) => x(),
        }
    }
}
