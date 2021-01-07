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
        if self.data_fetcher.is_some() {
            return Err(
                "Cannot set data as it is dynamically generated with a data fetcher".to_string(),
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn get_data_when_stored() {
        let data = "DATA".as_bytes().to_vec();

        let file_information = FileInformation {
            name: "".to_string(),
            data_fetcher: None,
            data: Some(data.clone()),
            listed: false,
            executable: false,
            writable: false,
            on_data_change: None,
        };
        assert_eq!(file_information.get_data(), data);
    }

    #[test]
    fn get_data_when_fetched() {
        let fetcher = Box::new(|| "DATA".as_bytes().to_vec());
        let expected = fetcher();

        let file_information = FileInformation {
            name: "".to_string(),
            data_fetcher: Some(fetcher),
            data: None,
            listed: false,
            executable: false,
            writable: false,
            on_data_change: None,
        };
        assert_eq!(file_information.get_data(), expected);
    }

    #[test]
    fn set_data_when_stored() {
        let (sender, receiver) = channel();
        let expected_data = "hello 123";

        let mut file_information = FileInformation {
            name: "".to_string(),
            data_fetcher: None,
            data: None,
            listed: false,
            executable: false,
            writable: false,
            on_data_change: Some(Box::new(move |received_data| {
                assert_eq!(received_data, expected_data);
                sender.send(()).unwrap();
                Ok(())
            })),
        };
        file_information
            .set_data(expected_data.as_bytes().to_vec())
            .unwrap();
        receiver.recv().unwrap();

        assert_eq!(
            std::str::from_utf8(file_information.get_data().as_slice()).unwrap(),
            expected_data
        );
    }

    #[test]
    fn set_data_when_fetched() {
        let mut file_information = FileInformation {
            name: "".to_string(),
            data_fetcher: Some(Box::new(|| "DATA".as_bytes().to_vec())),
            data: None,
            listed: false,
            executable: false,
            writable: false,
            on_data_change: None,
        };
        assert!(file_information
            .set_data("other".as_bytes().to_vec())
            .is_err());
    }
}
