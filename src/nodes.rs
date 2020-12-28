use fuse::{FileAttr, FileType};
use opencv::core::Vector;
use std::collections::HashMap;
use std::iter::Fuse;
use std::marker::PhantomData;
use std::time::SystemTime;
use users::{get_current_gid, get_current_uid};

pub const ROOT_INODE_NUMBER: u64 = 1;

pub fn create_directory_attributes(inode_number: u64) -> FileAttr {
    return FileAttr {
        ino: inode_number as u64,
        size: 0,
        blocks: 0,
        atime: SystemTime::UNIX_EPOCH,
        mtime: SystemTime::UNIX_EPOCH,
        ctime: SystemTime::UNIX_EPOCH,
        crtime: SystemTime::UNIX_EPOCH,
        kind: FileType::Directory,
        perm: 0o550,
        nlink: 1,
        uid: get_current_uid(),
        gid: get_current_gid(),
        rdev: 0,
        flags: 0,
    };
}

fn create_file_attributes(inode_number: u64, size: u64) -> FileAttr {
    return FileAttr {
        ino: inode_number,
        size: size,
        blocks: 1,
        atime: SystemTime::UNIX_EPOCH,
        mtime: SystemTime::UNIX_EPOCH,
        ctime: SystemTime::UNIX_EPOCH,
        crtime: SystemTime::UNIX_EPOCH,
        kind: FileType::RegularFile,
        perm: 0o440,
        nlink: 1,
        uid: get_current_uid(),
        gid: get_current_gid(),
        rdev: 0,
        flags: 0,
    };
}

pub enum FuseNode<'a> {
    Directory(&'a DirectoryFuseNode),
    File(&'a FrameFileFuseNode),
}

pub struct DirectoryFuseNode {
    pub attributes: FileAttr,
    pub name: String,
    pub parent_directory_inode_number: u64,
}

// TODO: move away from frame
pub struct FrameFileFuseNode {
    pub attributes: FileAttr,
    pub frame_id: u64,
    pub directory_inode_number: u64,
}

impl FrameFileFuseNode {
    pub(crate) fn get_name(&self) -> String {
        return format!("frame-{}.jpg", self.frame_id);
    }
}

pub struct FuseNodeStore<'a> {
    file_nodes: HashMap<u64, Box<FrameFileFuseNode>>,
    directory_nodes: HashMap<u64, Box<DirectoryFuseNode>>,
    // nodes: HashMap<u64, Box<dyn FuseNode<'a>>>,
    // XXX: it would be more efficient to store the references but hit lifetime issues...
    node_inode_numbers_in_directory: HashMap<u64, Vec<u64>>,
    current_inode_number: u64,
    phantom: PhantomData<&'a ()>,
}

impl<'a> FuseNodeStore<'a> {
    pub fn new() -> Self {
        let mut fuse_node_store = FuseNodeStore {
            file_nodes: Default::default(),
            directory_nodes: Default::default(),
            node_inode_numbers_in_directory: Default::default(),
            current_inode_number: ROOT_INODE_NUMBER,
            phantom: Default::default(),
        };
        fuse_node_store.insert_directory_with_inode_number(
            "root",
            ROOT_INODE_NUMBER,
            ROOT_INODE_NUMBER,
        );
        return fuse_node_store;
    }

    pub fn insert_directory(&mut self, name: &str, parent_directory_inode_number: u64) -> u64 {
        let inode_number = self.get_next_inode_number();
        return self.insert_directory_with_inode_number(
            name,
            inode_number,
            parent_directory_inode_number,
        );
    }

    pub fn insert_directory_with_inode_number(
        &mut self,
        name: &str,
        inode_number: u64,
        parent_directory_inode_number: u64,
    ) -> u64 {
        self.directory_nodes.insert(
            inode_number,
            Box::new(DirectoryFuseNode {
                attributes: create_directory_attributes(inode_number),
                name: name.to_string(),
                parent_directory_inode_number,
            }),
        );
        self.node_inode_numbers_in_directory
            .insert(inode_number, vec![]);
        self.link_to_directory(inode_number, parent_directory_inode_number);
        return inode_number;
    }

    pub fn insert_frame_file(
        &mut self,
        frame_id: u64,
        size: u64,
        directory_inode_number: u64,
    ) -> u64 {
        let inode_number = self.get_next_inode_number();
        self.file_nodes.insert(
            inode_number,
            Box::new(FrameFileFuseNode {
                attributes: create_file_attributes(inode_number, size),
                directory_inode_number,
                frame_id,
            }),
        );
        self.link_to_directory(inode_number, directory_inode_number);
        return inode_number;
    }

    pub fn get_file_node(&self, inode_number: u64) -> Option<&FrameFileFuseNode> {
        match self.file_nodes.get(&inode_number) {
            Some(boxed_node) => Some(boxed_node.as_ref()),
            None => None,
        }
    }

    pub fn get_directory_node(&self, inode_number: u64) -> Option<&DirectoryFuseNode> {
        match self.directory_nodes.get(&inode_number) {
            Some(boxed_node) => Some(boxed_node.as_ref()),
            None => None,
        }
    }

    pub fn get_node(&self, inode_number: u64) -> Option<FuseNode> {
        match self.get_file_node(inode_number) {
            Some(x) => Some(FuseNode::File(x)),
            None => match self.get_directory_node(inode_number) {
                Some(x) => Some(FuseNode::Directory(x)),
                None => None,
            },
        }
    }

    pub fn lookup_node(&self, name: &str, parent_directory_inode_number: u64) -> Option<FuseNode> {
        let children_inode_numbers = self
            .node_inode_numbers_in_directory
            .get(&parent_directory_inode_number)
            .expect(&format!(
                "Parent directory with inode number does not exist: {}",
                parent_directory_inode_number
            ));

        // TODO: specialised data structure to
        for child_inode_number in children_inode_numbers {
            match self.get_node(*child_inode_number) {
                Some(fuse_node) => {
                    let node_name = match fuse_node {
                        FuseNode::Directory(x) => x.name.clone(),
                        FuseNode::File(x) => x.get_name(),
                    };
                    if node_name == name {
                        return Some(fuse_node);
                    }
                }
                None => {}
            };
        }
        return None;
    }

    pub fn get_nodes_in_directory(&self, directory_inode_number: u64) -> Vec<FuseNode> {
        return self
            .node_inode_numbers_in_directory
            .get(&directory_inode_number)
            .expect(&format!(
                "Non-existent directory {}",
                directory_inode_number
            ))
            .iter()
            .map(|inode_number| {
                self.get_node(*inode_number).expect(&format!(
                    "Directory contains node that does not exist: {}",
                    inode_number
                ))
            })
            .collect();
    }

    fn get_next_inode_number(&mut self) -> u64 {
        self.current_inode_number += 1;
        return self.current_inode_number;
    }

    fn link_to_directory(&mut self, inode_number: u64, directory_inode_number: u64) {
        if !self.directory_nodes.contains_key(&directory_inode_number)
            && directory_inode_number != ROOT_INODE_NUMBER
        {
            panic!(
                "Directory to link to does not exist: {} (existing directories: {:?})",
                directory_inode_number,
                self.directory_nodes.keys()
            )
        }
        if self.get_node(inode_number).is_none() {
            panic!(
                "Node to link does not exist: {} (existing directories: {:?}; existing files: {:?})",
                inode_number,
                self.directory_nodes.keys(), self.file_nodes.keys()
            )
        }
        match self
            .node_inode_numbers_in_directory
            .get_mut(&directory_inode_number)
        {
            Some(nodes) => {
                nodes.push(inode_number);
            }
            None => {
                self.node_inode_numbers_in_directory
                    .insert(directory_inode_number, vec![inode_number]);
            }
        }
    }
}
