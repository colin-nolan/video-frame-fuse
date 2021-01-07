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

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::io::Cursor;

    #[test]
    fn output_when_none_added() {
        let manifest = DirectoryManifest::new();

        let serialised = manifest.to_vec();
        let mut reader = csv::Reader::from_reader(Cursor::new(&serialised));
        assert_eq!(
            &reader.headers().unwrap().iter().collect_vec(),
            DIRECTORY_MANIFEST_HEADER
        );
    }

    #[test]
    fn output_when_added() {
        let mut manifest = DirectoryManifest::new();
        manifest.add(ImageType::JPG, "1");
        manifest.add(ImageType::PNG, "2");

        let serialised = manifest.to_vec();
        let mut reader = csv::Reader::from_reader(Cursor::new(&serialised));
        assert_eq!(
            &reader.headers().unwrap().iter().collect_vec(),
            DIRECTORY_MANIFEST_HEADER
        );

        assert_eq!(
            reader
                .records()
                .map(|x| x.unwrap().iter().map(str::to_string).collect_vec())
                .collect_vec()
                .sort(),
            vec![
                format!("{},1", ImageType::JPG.to_string()),
                format!("{},2", ImageType::PNG.to_string())
            ]
            .sort()
        );
    }
}
