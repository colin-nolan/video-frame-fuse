use fuse::{FileAttr, FileType};
use opencv::core::Vector;
use std::collections::HashMap;
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
        perm: 0o500,
        nlink: 1,
        uid: get_current_uid(),
        gid: get_current_gid(),
        rdev: 0,
        flags: 0,
    };
}

pub trait FuseNode<'a> {
    fn get_attributes(&self) -> FileAttr;

    fn get_name(&self) -> String;
}

pub struct DirectoryFuseNode {
    attributes: FileAttr,
    name: String,
    parent_directory_inode_number: u64,
}

impl<'a> FuseNode<'a> for DirectoryFuseNode {
    fn get_attributes(&self) -> FileAttr {
        return self.attributes;
    }

    fn get_name(&self) -> String {
        return self.name.to_string();
    }
}

pub struct FrameFileFuseNode {
    attributes: FileAttr,
    frame_id: u64,
    directory_inode_number: u64,
}

impl<'a> FuseNode<'a> for FrameFileFuseNode {
    fn get_attributes(&self) -> FileAttr {
        return self.attributes;
    }

    fn get_name(&self) -> String {
        return format!("frame-{}.jpg", self.frame_id);
    }
}

pub struct FuseNodeStore<'a> {
    nodes: HashMap<u64, Box<dyn FuseNode<'a>>>,
    // XXX: it would be more efficient to store the references but hit lifetime issues...
    node_inode_numbers_in_directory: HashMap<u64, Vec<u64>>,
    current_inode_number: u64,
}

impl<'a> FuseNodeStore<'a> {
    pub fn new() -> Self {
        let mut fuse_node_store = FuseNodeStore {
            nodes: Default::default(),
            node_inode_numbers_in_directory: Default::default(),
            current_inode_number: ROOT_INODE_NUMBER,
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
        self.nodes.insert(
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

    pub fn insert_frame_file(&mut self, frame_id: u64, directory_inode_number: u64) -> u64 {
        let inode_number = self.get_next_inode_number();
        self.nodes.insert(
            inode_number,
            Box::new(FrameFileFuseNode {
                attributes: create_directory_attributes(inode_number),
                directory_inode_number,
                frame_id,
            }),
        );
        self.link_to_directory(inode_number, directory_inode_number);
        return inode_number;
    }

    pub fn get_node(&self, inode_number: u64) -> Option<&dyn FuseNode<'a>> {
        return match self.nodes.get(&inode_number) {
            Some(boxed_fuse_node) => Some(boxed_fuse_node.as_ref()),
            None => None,
        };
    }

    pub fn lookup_node(
        &self,
        name: &str,
        parent_directory_inode_number: u64,
    ) -> Option<&dyn FuseNode<'a>> {
        let children_inode_numbers = self
            .node_inode_numbers_in_directory
            .get(&parent_directory_inode_number)
            .expect(&format!(
                "Parent directory with inode number does not exist: {}",
                parent_directory_inode_number
            ));

        // TODO: specialised data structure to
        for child_inode_number in children_inode_numbers {
            let node = self.get_node(*child_inode_number);
            if node.is_some() && node.unwrap().get_name() == name {
                eprintln!("Found: {}", node.unwrap().get_name());
                return node;
            }
        }
        return None;
    }

    pub fn get_nodes_in_directory(&self, directory_inode_number: u64) -> Vec<&dyn FuseNode<'a>> {
        return self
            .node_inode_numbers_in_directory
            .get(&directory_inode_number)
            .expect(&format!(
                "Non-existent directory {}",
                directory_inode_number
            ))
            .iter()
            .map(|inode_number| {
                self.nodes
                    .get(inode_number)
                    .expect(&format!(
                        "Directory contains node that does not exist: {}",
                        inode_number
                    ))
                    .as_ref()
            })
            .collect();
    }

    fn get_next_inode_number(&mut self) -> u64 {
        self.current_inode_number += 1;
        return self.current_inode_number;
    }

    fn link_to_directory(&mut self, inode_number: u64, directory_inode_number: u64) {
        if !self.nodes.contains_key(&directory_inode_number)
            && directory_inode_number != ROOT_INODE_NUMBER
        {
            panic!(
                "Directory to link to does not exist: {} (existing directories: {:?})",
                directory_inode_number,
                self.nodes.keys()
            )
        }
        if !self.nodes.contains_key(&inode_number) {
            panic!(
                "Node to link does not exist: {} (existing directories: {:?})",
                inode_number,
                self.nodes.keys()
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
