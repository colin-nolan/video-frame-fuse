use crate::frames;
use crate::frames::{close_video, frame_to_jpg, get_number_of_frames, open_video};
use crate::nodes;
use crate::nodes::{create_directory_attributes, FuseNodeStore, ROOT_INODE_NUMBER};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use opencv::videoio::VideoCapture;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::time::{Duration, SystemTime};
use users::{get_current_gid, get_current_uid};

const ENOENT: i32 = 2;

const TTL: Duration = Duration::from_secs(1);

lazy_static! {
    static ref ROOT_DIRECTORY_ATTRIBUTES: FileAttr = create_directory_attributes(1);
}

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 1089132,
    blocks: 1,
    atime: SystemTime::UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: SystemTime::UNIX_EPOCH,
    ctime: SystemTime::UNIX_EPOCH,
    crtime: SystemTime::UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

pub struct VideoFileSystem<'a> {
    video: VideoCapture,
    nodes: FuseNodeStore<'a>,
}

impl<'a> VideoFileSystem<'a> {
    pub fn new(video_location: &str) -> Self {
        let video = open_video(video_location);
        let number_of_frames = get_number_of_frames(&video);

        let mut node_store = FuseNodeStore::new();
        // let root_directory_inode_number = node_store.insert_directory("root", ROOT_INODE_NUMBER);
        let by_frame_number_inode_number =
            node_store.insert_directory("by_frame", ROOT_INODE_NUMBER);

        // TODO: attach frame images

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
        eprintln!("lookup: {}, {}", parent, name.to_str().unwrap());

        match self.nodes.lookup_node(
            name.to_str().expect("Could not convert OsStr to string"),
            parent,
        ) {
            Some(fuse_node) => {
                reply.entry(&TTL, &fuse_node.get_attributes(), 0);
            }
            None => {
                eprintln!(
                    "No node: name={}, parent={}",
                    name.to_str().unwrap(),
                    parent
                );
                reply.error(ENOENT);
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        eprintln!("getattr: {}", ino);

        match self.nodes.get_node(ino) {
            Some(fuse_node) => {
                eprintln!("Returning attributes for {}", ino);
                reply.attr(&TTL, &fuse_node.get_attributes());
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
        eprintln!("read");

        if ino == 2 {
            let frame = frames::get_frame(0, &mut self.video);

            let data = &mut Default::default();
            frame_to_jpg(&frame, data);

            eprintln!("number_frames: {}", get_number_of_frames(&self.video));
            eprintln!("frame_data.len: {}", data.len());
            eprintln!("data.len: {}", data.len());
            eprintln!("offset: {}", offset);
            eprintln!("size: {}", size);

            // let custom_bytes = [155, 255, 87];
            // reply.data(&custom_bytes[offset as usize..]);

            reply.data(&data.as_slice()[offset as usize..(offset as usize + size as usize)]);

            eprintln!("Data replied")
        } else {
            eprintln!("error ENOENT");
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        eprintln!("readdir, ino={}", ino);

        let node = self.nodes.get_node(ino);
        if node.is_none() {
            eprintln!("No node");
            reply.error(ENOENT);
            return;
        }

        // let entries = vec![
        //     (1, FileType::Directory, "."),
        //     (1, FileType::Directory, ".."),
        //     (2, FileType::RegularFile, "frame0.jpg"),
        // ];

        let mut entries = vec![
            (ino, FileType::Directory, ".".to_string()),
            // FIXME: inode needs to be correct as directory
            (ino, FileType::Directory, "..".to_string()),
        ];
        entries.extend(
            self.nodes
                .get_nodes_in_directory(ino)
                .into_iter()
                .map(|node| {
                    let attributes = node.get_attributes();
                    println!("Added: {}", node.get_name());
                    (
                        attributes.ino as u64,
                        attributes.kind,
                        node.get_name().clone(),
                    )
                }),
        );

        println!("{} entries (offset: {})", entries.len(), offset);
        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            println!("Adding to readdir: {}, {}", entry.0, entry.2);
            // i + 1 means the index of the next entry
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        println!("End read entries");
        reply.ok();
    }
}
