use crate::fuse_fs::models::nodes::{create_directory_attributes, FuseNode, FuseNodeStore};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyWrite,
    Request,
};
use libc::{EIO, ENOENT, EPERM};
use log::{debug, error};
use std::cmp::{max, min};
use std::ffi::OsStr;
use std::time::{Duration, SystemTime};

const TTL: Duration = Duration::from_secs(1);

lazy_static! {
    static ref ROOT_DIRECTORY_ATTRIBUTES: FileAttr = create_directory_attributes(1);
}

pub struct VideoFileSystem<'a> {
    pub nodes: FuseNodeStore<'a>,
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
                    FuseNode::File(file_node) => {
                        requires_listing = !file_node.information.listed;
                        file_node.get_attributes()
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
        match self.nodes.get_file_node(inode_number) {
            None => {
                error!("Node not found (setattr): {}", inode_number);
                reply.error(EIO);
            }
            Some(node) => reply.attr(&TTL, &node.get_attributes()),
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
        let node = match self.nodes.get_file_node_mut(ino) {
            Some(file_node) => file_node,
            None => {
                error!("Node not found (read): {}", ino);
                reply.error(ENOENT);
                return;
            }
        };

        let data = node.get_data();
        let end = min(offset as usize + size as usize, data.len());
        reply.data(&data[offset as usize..end]);

        node.information.listed = true;
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        inode_number: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        debug!(
            "Writing data of length {} to {} with offset {}",
            data.len(),
            offset,
            inode_number,
        );
        let node_result = self.nodes.get_file_node_mut(inode_number);
        if node_result.is_none() {
            error!("Node not found (write) {}", inode_number);
            reply.error(EIO);
            return;
        }
        let node = node_result.unwrap();

        let mut data_as_vec;
        if offset == 0 {
            data_as_vec = data.to_vec();
        } else {
            data_as_vec = node.information.get_data();
            let end_position: usize = offset as usize + data.len();
            // XXX: there is likely a more efficient way to support an append operation (common)
            data_as_vec.resize(max(data_as_vec.len(), end_position), 0);
            data_as_vec.splice(offset as usize..end_position, data.to_vec());
        }

        // FIXME: setting the data calls the callback, even though the full data may not be complete
        let write_result = node.information.set_data(data_as_vec);
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
}
