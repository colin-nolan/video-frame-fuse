use crate::configuration::{BlackAndWhiteConfiguration, Configuration, ConfigurationHolder};
use crate::file_system::data::FileInformation;
use crate::file_system::manifest::DirectoryManifest;
use crate::file_system::nodes::{
    create_directory_attributes, DirectoryFuseNode, FuseNode, FuseNodeStore,
};
use crate::video_processing::{
    get_black_and_white_frame_image, get_frame_image, get_greyscale_frame_image,
    get_number_of_frames, open_video, ImageType,
};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyWrite,
    Request,
};
use libc::{ENOENT, EPERM};
use log::error;
use opencv::videoio::VideoCapture;
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use strum::IntoEnumIterator;

const TTL: Duration = Duration::from_secs(1);

lazy_static! {
    static ref ROOT_DIRECTORY_ATTRIBUTES: FileAttr = create_directory_attributes(1);
}

pub struct VideoFileSystem<'a> {
    video: VideoCapture,
    nodes: FuseNodeStore<'a>,
}

impl<'a> VideoFileSystem<'a> {
    fn create_frame_view(
        directory_name: &str,
        video_location: &'static str,
        frame_number: u64,
        directory_attributes_generator: &mut dyn FnMut() -> FileAttr,
        image_data_generator: &'static dyn Fn(
            // TODO: ref?
            String,
            u64,
            ImageType,
            ConfigurationHolder,
        ) -> Vec<u8>,
        // Does this actually need to be boxed?
        configuration_parser: Option<
            Box<&'static dyn Fn(&str) -> Result<ConfigurationHolder, String>>,
        >,
        default_configuration: ConfigurationHolder,
    ) -> DirectoryFuseNode {
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

                    file_informations.push(FileInformation::new(
                        &file_name,
                        Box::new(move || {
                            image_data_generator(
                                video_location.to_string(),
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
                    include_bytes!("../resources/initialise.sh").to_vec(),
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
                        let mut configuration_holder =
                            movable_configuration_holder.write().unwrap();
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

    pub fn new(video_location: &'static str) -> Self {
        let mut video = open_video(video_location);

        let mut node_store = FuseNodeStore::new();
        let root_directory_inode_number = node_store.get_root_directory().get_inode_number();
        let by_frame_directory_inode_number =
            node_store.create_and_insert_directory("by-frame", root_directory_inode_number);

        let number_of_frames = get_number_of_frames(&mut video);
        for frame_number in 1..number_of_frames as u64 {
            let frame_directory_inode_number = node_store.create_and_insert_directory(
                &format!("frame-{}", frame_number),
                by_frame_directory_inode_number,
            );

            let originals_directory = VideoFileSystem::create_frame_view(
                "original",
                video_location,
                frame_number,
                &mut || create_directory_attributes(node_store.create_inode_number()),
                &|video_location, frame_number, image_type, configuration_holder| {
                    get_frame_image(video_location, frame_number, image_type)
                },
                None,
                ConfigurationHolder::None,
            );
            // TODO: factory that creates and inserts
            node_store.insert_directory(originals_directory, frame_directory_inode_number);

            let greyscales_directory = VideoFileSystem::create_frame_view(
                "greyscale",
                video_location,
                frame_number,
                &mut || create_directory_attributes(node_store.create_inode_number()),
                &|video_location, frame_number, image_type, configuration_holder| {
                    get_greyscale_frame_image(video_location, frame_number, image_type)
                },
                None,
                ConfigurationHolder::None,
            );
            // TODO: factory that creates and inserts
            node_store.insert_directory(greyscales_directory, frame_directory_inode_number);

            let black_and_white_directory = VideoFileSystem::create_frame_view(
                "black-and-white",
                video_location,
                frame_number,
                &mut || create_directory_attributes(node_store.create_inode_number()),
                &|video_location, frame_number, image_type, configuration_holder| {
                    let threshold = match configuration_holder {
                        ConfigurationHolder::BlackAndWhite(x) => x.threshold,
                        // TODO: default should already exist?
                        ConfigurationHolder::None => 128,
                    };
                    get_black_and_white_frame_image(
                        video_location,
                        frame_number,
                        threshold,
                        image_type,
                    )
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

        // TODO: need video?
        return VideoFileSystem {
            video,
            nodes: node_store,
        };

        // TODO: close video regardless of failure
        // close_video(video);
    }
}

impl Filesystem for VideoFileSystem<'_> {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name = name.to_str().expect("Could not convert OsStr to string");

        let mut requires_listing = false;
        let inode_number;
        let node = self.nodes.lookup_node(name, parent);
        match node {
            Some(fuse_node) => {
                let attributes = match fuse_node {
                    FuseNode::Directory(x) => x.attributes,
                    FuseNode::File(x) => {
                        requires_listing = !x.information.listed;
                        x.get_attributes()
                    }
                };
                inode_number = attributes.ino;
                reply.entry(&TTL, &attributes, 0);
            }
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        // Note: we cannot change the status in the above match because we are already borrowing
        //       mut (mut because the directory listing may be generated on call).
        if requires_listing {
            self.nodes
                .get_file_node_mut(inode_number)
                .expect(&format!(
                    "Could not get mutable copy of file node: {}",
                    inode_number
                ))
                .information
                .listed = true;
        }
    }

    fn getattr(&mut self, _req: &Request, inode_number: u64, reply: ReplyAttr) {
        match self.nodes.get_node(inode_number) {
            Some(fuse_node) => {
                let attributes = match fuse_node {
                    FuseNode::Directory(x) => x.attributes,
                    FuseNode::File(x) => x.get_attributes(),
                };
                reply.attr(&TTL, &attributes);
            }
            None => {
                eprintln!("No node (getattr): {:?}", inode_number);
                reply.error(ENOENT);
            }
        };
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        let node = match self.nodes.get_file_node_mut(ino) {
            Some(x) => x,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let data = node.get_data();
        reply.data(&data[offset as usize..offset as usize + size as usize]);

        node.information.listed = true;
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let node = self.nodes.get_directory_node(ino);
        if node.is_none() {
            reply.error(ENOENT);
            return;
        }

        let mut entries = vec![
            (ino, FileType::Directory, ".".to_string()),
            (ino, FileType::Directory, "..".to_string()),
        ];
        entries.extend(
            self.nodes
                .get_nodes_in_directory(ino)
                .into_iter()
                .filter(|fuse_node| match fuse_node {
                    FuseNode::Directory(_) => true,
                    FuseNode::File(x) => x.information.listed,
                })
                .map(|fuse_node| {
                    let attributes;
                    let name;
                    match fuse_node {
                        FuseNode::Directory(x) => {
                            attributes = x.attributes;
                            name = x.name.to_string();
                        }
                        FuseNode::File(x) => {
                            attributes = x.get_attributes();
                            name = x.information.name.to_string();
                        }
                    };
                    (attributes.ino as u64, attributes.kind, name)
                }),
        );

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        reply.ok();
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        inode_number: u64,
        _fh: u64,
        _offset: i64,
        data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        let node = self
            .nodes
            .get_file_node_mut(inode_number)
            // TODO: change to reply.error
            .expect(&format!("Could not fetch node: {}", inode_number));
        // FIXME: consider offset...
        let write_result = node.information.set_data(data.to_vec());
        if write_result.is_err() {
            error!(
                "Error writing file \"{}\": {}",
                node.information.name,
                write_result.unwrap_err()
            );
            reply.error(EPERM);
        } else {
            reply.written(data.len() as u32);
        }
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        inode_number: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<SystemTime>,
        _mtime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        // TODO: consider other attributes
        let node = self
            .nodes
            .get_file_node(inode_number)
            // TODO: change to reply.error
            .expect(&format!("Could not fetch node: {}", inode_number));
        reply.attr(&TTL, &node.get_attributes());
    }
}
