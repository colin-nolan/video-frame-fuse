use crate::frames;
use crate::frames::ImageType::{JPG, PNG};
use crate::frames::{
    close_video, convert_frame, get_frame, get_frame_from_video, get_number_of_frames, open_video,
    read_frame, FileInformation, ImageType,
};
use crate::manifest::DirectoryManifest;
use crate::nodes::{
    create_directory_attributes, create_file_attributes, DirectoryFuseNode, FileFuseNode, FuseNode,
    FuseNodeStore,
};
use csv::Writer;
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use opencv::core::error;
use opencv::videoio::VideoCapture;
use std::ffi::OsStr;
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const ENOENT: i32 = 2;

const TTL: Duration = Duration::from_secs(1);

lazy_static! {
    static ref ROOT_DIRECTORY_ATTRIBUTES: FileAttr = create_directory_attributes(1);
}

pub struct VideoFileSystem<'a> {
    video: VideoCapture,
    nodes: FuseNodeStore<'a>,
}

impl<'a> VideoFileSystem<'a> {
    pub fn new(video_location: &'static str) -> Self {
        let mut video = open_video(video_location);

        let mut node_store = FuseNodeStore::new();
        let root_directory_inode_number = node_store.get_root_directory().get_inode_number();
        let by_frame_directory_inode_number =
            node_store.create_and_insert_directory("by-frame", root_directory_inode_number);

        let number_of_frames = get_number_of_frames(&mut video);
        for frame_number in 1..number_of_frames as u64 {
            let frame_name = format!("frame-{}", frame_number);

            let directory = DirectoryFuseNode::new(
                frame_name.clone(),
                create_directory_attributes(node_store.create_inode_number()),
                Box::new(move |directory_inode_number| {
                    let mut directory_manifest = DirectoryManifest::new();

                    let mut file_informations: Vec<FileInformation> = ImageType::iter()
                        .map(|image_type: ImageType| {
                            let name = format!("{}.{}", frame_name, image_type.to_string());
                            directory_manifest.add(image_type, &name);
                            FileInformation::new(
                                &name,
                                Box::new(move || {
                                    read_frame(video_location, frame_number, image_type)
                                }),
                                false,
                                false,
                            )
                        })
                        .collect();

                    file_informations.push(FileInformation::new_with_data(
                        "manifest.csv",
                        directory_manifest.to_vec(),
                        true,
                        false,
                    ));

                    file_informations.push(FileInformation::new_with_data(
                        "initialise.sh",
                        include_bytes!("../resources/initialise.sh").to_vec(),
                        true,
                        true,
                    ));

                    return file_informations;
                }),
            );
            node_store.insert_directory(directory, by_frame_directory_inode_number);
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
        let mut inode_number;
        let node = self.nodes.lookup_node(name, parent);
        match node {
            Some(fuse_node) => {
                let attributes = match fuse_node {
                    FuseNode::Directory(x) => x.attributes,
                    FuseNode::File(x) => {
                        requires_listing = !x.listed;
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

        node.listed = true;
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
                    FuseNode::File(x) => x.listed,
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
                            name = x.name.to_string();
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
}
