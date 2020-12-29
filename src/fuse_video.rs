use crate::frames;
use crate::frames::{close_video, frame_to_jpg, get_frame, get_number_of_frames, open_video};
use crate::nodes::{create_directory_attributes, DirectoryFuseNode, FuseNode, FuseNodeStore};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use opencv::videoio::VideoCapture;
use std::ffi::OsStr;
use std::time::Duration;

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
    pub fn new(video_location: &str) -> Self {
        let mut video = open_video(video_location);

        let mut node_store = FuseNodeStore::new();
        let root_directory_inode_number = node_store.get_root_directory().get_inode_number();
        let by_frame_directory_inode_number =
            node_store.create_and_insert_directory("by-frame", root_directory_inode_number);

        let number_of_frames = get_number_of_frames(&mut video);
        for frame_number in 1..number_of_frames as u64 {
            let directory = DirectoryFuseNode::new(
                &format!("frame-{}", frame_number),
                create_directory_attributes(node_store.create_inode_number()),
                Box::new(|| vec![]),
            );
            node_store.insert_directory(directory, by_frame_directory_inode_number);
        }

        return VideoFileSystem {
            video,
            nodes: node_store,
        };

        // TODO: close video regardless of failure
        // close_video(video);
    }

    fn get_frame_jpg(&mut self, frame_number: u64) -> Vec<u8> {
        let frame = get_frame(frame_number, &mut self.video);
        return frame_to_jpg(&frame);
    }
}

impl Filesystem for VideoFileSystem<'_> {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name = name.to_str().expect("Could not convert OsStr to string");
        match self.nodes.lookup_node(name, parent) {
            Some(fuse_node) => {
                let attributes = match fuse_node {
                    FuseNode::Directory(x) => x.attributes,
                    FuseNode::File(x) => x.attributes,
                };
                reply.entry(&TTL, &attributes, 0);
            }
            None => {
                eprintln!("No node: name={}, parent={}", name, parent);
                reply.error(ENOENT);
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.nodes.get_node(ino) {
            Some(fuse_node) => {
                let attributes = match fuse_node {
                    FuseNode::Directory(x) => x.attributes,
                    FuseNode::File(x) => x.attributes,
                };
                reply.attr(&TTL, &attributes);
            }
            None => {
                eprintln!("No node (getattr): {:?}", ino);
                reply.error(ENOENT);
            }
        }
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
        let node = match self.nodes.get_file_node(ino) {
            Some(x) => x,
            None => {
                eprintln!("No file node");
                reply.error(ENOENT);
                return;
            }
        };

        let frame = get_frame(node.frame_id, &mut self.video);
        let frame_as_jpg = frame_to_jpg(&frame);

        reply.data(&frame_as_jpg[offset as usize..offset as usize + size as usize]);

        eprintln!("read: {}", ino);

        // if ino == 2 {
        //     let frame = frames::get_frame(0, &mut self.video);
        //
        //     let data = frame_to_jpg(&frame);
        //
        //     eprintln!("number_frames: {}", get_number_of_frames(&self.video));
        //     eprintln!("frame_data.len: {}", data.len());
        //     eprintln!("data.len: {}", data.len());
        //     eprintln!("offset: {}", offset);
        //     eprintln!("size: {}", size);
        //
        //     // let custom_bytes = [155, 255, 87];
        //     // reply.data(&custom_bytes[offset as usize..]);
        //
        //
        //
        //     eprintln!("Data replied")
        // } else {
        //     eprintln!("error ENOENT");
        //     reply.error(ENOENT);
        // }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let node = self.nodes.get_node(ino);
        if node.is_none() {
            eprintln!("No node");
            reply.error(ENOENT);
            return;
        }
        // TODO: ensure directory

        let mut entries = vec![
            (ino, FileType::Directory, ".".to_string()),
            (ino, FileType::Directory, "..".to_string()),
        ];
        entries.extend(
            self.nodes
                .get_nodes_in_directory(ino)
                .into_iter()
                .map(|fuse_node| {
                    let attributes;
                    let name;
                    match fuse_node {
                        FuseNode::Directory(x) => {
                            attributes = x.attributes;
                            name = x.name.clone();
                        }
                        FuseNode::File(x) => {
                            attributes = x.attributes;
                            name = x.get_name();
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
