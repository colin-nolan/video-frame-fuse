use crate::fuse_fs::models::config::{
    BlackAndWhiteConfiguration, Configuration, ConfigurationHolder,
};
use crate::fuse_fs::models::file::FileInformation;
use crate::fuse_fs::models::manifest::DirectoryManifest;
use crate::fuse_fs::models::nodes::{
    create_directory_attributes, DirectoryFuseNode, FuseNodeStore,
};
use crate::video_processing::{
    get_black_and_white_frame_image, get_frame_image, get_greyscale_frame_image,
    get_number_of_frames, ImageType,
};
use fuse::FileAttr;
use std::sync::{Arc, RwLock};
use strum::IntoEnumIterator;

pub fn create_video_nodes(video_location: &str) -> FuseNodeStore {
    let mut node_store = FuseNodeStore::new();
    let root_directory_inode_number = node_store.get_root_directory().get_inode_number();
    let by_frame_directory_inode_number =
        node_store.create_and_insert_directory("by-frame", root_directory_inode_number);

    let number_of_frames = get_number_of_frames(video_location);
    for frame_number in 1..number_of_frames as u64 {
        let frame_directory_inode_number = node_store.create_and_insert_directory(
            &format!("frame-{}", frame_number),
            by_frame_directory_inode_number,
        );

        let originals_directory = create_frame_view(
            "original",
            video_location,
            frame_number,
            &mut || create_directory_attributes(node_store.create_inode_number()),
            &|video_location, frame_number, image_type, _| {
                get_frame_image(video_location, frame_number, image_type)
            },
            None,
            ConfigurationHolder::None,
        );
        // TODO: factory that creates and inserts
        node_store.insert_directory(originals_directory, frame_directory_inode_number);

        let greyscales_directory = create_frame_view(
            "greyscale",
            video_location,
            frame_number,
            &mut || create_directory_attributes(node_store.create_inode_number()),
            &|video_location, frame_number, image_type, _| {
                get_greyscale_frame_image(video_location, frame_number, image_type)
            },
            None,
            ConfigurationHolder::None,
        );
        // TODO: factory that creates and inserts
        node_store.insert_directory(greyscales_directory, frame_directory_inode_number);

        let black_and_white_directory = create_frame_view(
            "black-and-white",
            video_location,
            frame_number,
            &mut || create_directory_attributes(node_store.create_inode_number()),
            &|video_location, frame_number, image_type, configuration_holder| {
                let threshold = match configuration_holder {
                    ConfigurationHolder::BlackAndWhite(x) => x.threshold,
                    _ => panic!("Incorrect configuration type"),
                };
                get_black_and_white_frame_image(video_location, frame_number, threshold, image_type)
            },
            Some(Box::new(
                &|data| match BlackAndWhiteConfiguration::from_yaml(data) {
                    Ok(x) => Ok(ConfigurationHolder::BlackAndWhite(x)),
                    Err(e) => Err(e),
                },
            )),
            ConfigurationHolder::BlackAndWhite(BlackAndWhiteConfiguration::default()),
        );
        // TODO: factory that creates and inserts
        node_store.insert_directory(black_and_white_directory, frame_directory_inode_number);
    }

    return node_store;
}

fn create_frame_view(
    directory_name: &str,
    video_location: &str,
    frame_number: u64,
    directory_attributes_generator: &mut dyn FnMut() -> FileAttr,
    image_data_generator: &'static dyn Fn(String, u64, ImageType, ConfigurationHolder) -> Vec<u8>,
    // Does this actually need to be boxed?
    configuration_parser: Option<Box<&'static dyn Fn(&str) -> Result<ConfigurationHolder, String>>>,
    default_configuration: ConfigurationHolder,
) -> DirectoryFuseNode {
    let video_location = video_location.to_string();

    DirectoryFuseNode::new(
        directory_name,
        directory_attributes_generator(),
        Box::new(move |_| {
            let mut directory_manifest = DirectoryManifest::new();
            let mut file_informations = vec![];
            let configuration_holder = Arc::new(RwLock::new(default_configuration.to_owned()));

            for image_type in ImageType::iter() {
                let file_name = format!("frame-{}.{}", frame_number, image_type.to_string());
                let movable_configuration_holder = configuration_holder.clone();
                let movable_video_location = video_location.to_string();

                file_informations.push(FileInformation::new(
                    &file_name,
                    Box::new(move || {
                        image_data_generator(
                            movable_video_location.to_string(),
                            frame_number,
                            image_type,
                            movable_configuration_holder.read().unwrap().clone(),
                        )
                    }),
                    false,
                    false,
                ));
                directory_manifest.add(image_type, &file_name);
            }

            file_informations.push(FileInformation::new_with_data(
                "manifest.csv",
                directory_manifest.to_vec(),
                true,
                false,
                false,
                None,
            ));

            file_informations.push(FileInformation::new_with_data(
                "initialise.sh",
                include_bytes!("../../resources/initialise.sh").to_vec(),
                true,
                true,
                false,
                None,
            ));

            // Required to use within inner closure
            let configuration_parser = configuration_parser.clone();
            let movable_configuration_holder = configuration_holder.clone();
            // TODO: correctly handle unwrap
            let config_change_handler: Option<Box<dyn Fn(&str) -> Result<(), String>>> =
                Some(Box::new(move |data| {
                    // TODO: handle invalid config, don't just unwrap!
                    let configuration = configuration_parser.as_ref().unwrap()(data)?;

                    // Update configuration shared with data generators
                    let mut configuration_holder = movable_configuration_holder.write().unwrap();
                    eprintln!("Updating shared configuration...");
                    *configuration_holder = configuration;
                    Ok(())
                }));

            file_informations.push(FileInformation::new_with_data(
                "config.yml",
                match &default_configuration {
                    ConfigurationHolder::BlackAndWhite(x) => x.to_yaml().unwrap().into_bytes(),
                    _ => vec![],
                },
                true,
                false,
                true,
                config_change_handler,
            ));

            return file_informations;
        }),
    )
}
