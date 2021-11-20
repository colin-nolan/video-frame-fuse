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
use log::{debug, info};
use std::sync::{Arc, RwLock};
use strum::IntoEnumIterator;

lazy_static! {
    static ref DEFAULT_VIEW_GENERATORS: Vec<fn(&str, u64, u64) -> DirectoryFuseNode> = vec![
        create_original_view,
        create_greyscale_view,
        create_black_and_white_view,
    ];
}

pub fn create_default_video_nodes(video_location: &str) -> FuseNodeStore {
    create_video_nodes(video_location, DEFAULT_VIEW_GENERATORS.to_vec())
}

pub fn create_video_nodes(
    video_location: &str,
    view_generators: Vec<fn(&str, u64, u64) -> DirectoryFuseNode>,
) -> FuseNodeStore {
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

        for view_generator in &view_generators {
            let view_directory = view_generator(
                video_location,
                frame_number,
                node_store.create_inode_number(),
            );
            node_store.insert_directory(view_directory, frame_directory_inode_number);
        }
    }

    node_store
}

pub fn create_original_view(
    video_location: &str,
    frame_number: u64,
    inode_number: u64,
) -> DirectoryFuseNode {
    create_frame_view(
        "original",
        video_location,
        frame_number,
        &mut || create_directory_attributes(inode_number),
        &|video_location, frame_number, image_type, _| {
            get_frame_image(video_location, frame_number, image_type)
        },
        None,
        ConfigurationHolder::None,
    )
}

pub fn create_greyscale_view(
    video_location: &str,
    frame_number: u64,
    inode_number: u64,
) -> DirectoryFuseNode {
    create_frame_view(
        "greyscale",
        video_location,
        frame_number,
        &mut || create_directory_attributes(inode_number),
        &|video_location, frame_number, image_type, _| {
            get_greyscale_frame_image(video_location, frame_number, image_type)
        },
        None,
        ConfigurationHolder::None,
    )
}

pub fn create_black_and_white_view(
    video_location: &str,
    frame_number: u64,
    inode_number: u64,
) -> DirectoryFuseNode {
    create_frame_view(
        "black-and-white",
        video_location,
        frame_number,
        &mut || create_directory_attributes(inode_number),
        &|video_location, frame_number, image_type, configuration_holder| {
            let threshold = match configuration_holder {
                ConfigurationHolder::BlackAndWhite(x) => x.threshold,
                _ => panic!("Incorrect configuration type"),
            };
            get_black_and_white_frame_image(video_location, frame_number, threshold, image_type)
        },
        Some(&|data| match BlackAndWhiteConfiguration::from_yaml(data) {
            Ok(x) => Ok(ConfigurationHolder::BlackAndWhite(x)),
            Err(e) => Err(e),
        }),
        ConfigurationHolder::BlackAndWhite(BlackAndWhiteConfiguration::default()),
    )
}

pub fn create_frame_view(
    view_name: &str,
    video_location: &str,
    frame_number: u64,
    directory_attributes_generator: &mut dyn FnMut() -> FileAttr,
    image_data_generator: &'static dyn Fn(String, u64, ImageType, ConfigurationHolder) -> Vec<u8>,
    configuration_parser: Option<&'static dyn Fn(&str) -> Result<ConfigurationHolder, String>>,
    default_configuration: ConfigurationHolder,
) -> DirectoryFuseNode {
    let video_location = video_location.to_string();
    let view_name = view_name.to_string();

    DirectoryFuseNode::new(
        &(view_name.clone()),
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
            let movable_view_name = view_name.to_string();
            // TODO: correctly handle unwrap
            let config_change_handler: Option<Box<dyn Fn(&str) -> Result<(), String>>> =
                Some(Box::new(move |data| {
                    debug!("Received updated configuration: {}", data);

                    // TODO: handle invalid config, don't just unwrap!
                    let configuration = configuration_parser.as_ref().unwrap()(data)?;

                    // Update configuration shared with data generators
                    let mut configuration_holder = movable_configuration_holder.write().unwrap();
                    *configuration_holder = configuration.clone();
                    info!(
                        "Updated {} configuration for frame {}: {:?}",
                        movable_view_name, frame_number, configuration
                    );
                    Ok(())
                }));

            file_informations.push(FileInformation::new_with_data(
                "config.yml",
                match &default_configuration {
                    ConfigurationHolder::BlackAndWhite(configuration) => {
                        configuration.to_yaml().unwrap().into_bytes()
                    }
                    _ => vec![],
                },
                true,
                false,
                true,
                config_change_handler,
            ));

            file_informations
        }),
    )
}
