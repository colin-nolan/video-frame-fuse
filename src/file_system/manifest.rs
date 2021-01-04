use crate::video_processing::ImageType;
use csv::Writer;

const DIRECTORY_MANIFEST_HEADER: &[&str; 2] = &["image-type", "location"];

pub struct DirectoryManifest {
    records: Vec<[String; 2]>,
}

impl DirectoryManifest {
    pub fn new() -> Self {
        DirectoryManifest { records: vec![] }
    }

    pub fn add(&mut self, image_type: ImageType, location: &str) {
        self.records
            .push([image_type.to_string(), location.to_string()])
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut writer = Writer::from_writer(vec![]);
        writer.write_record(DIRECTORY_MANIFEST_HEADER).unwrap();
        for record in &self.records {
            writer.write_record(record).unwrap();
        }
        return writer.into_inner().unwrap();
    }
}
