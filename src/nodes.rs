use fuse::{FileAttr, FileType};
use std::borrow::Borrow;
use std::collections::HashMap;
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
        size,
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

#[derive(Clone)]
pub enum FuseNode<'a> {
    Directory(&'a DirectoryFuseNode),
    File(&'a FrameFileFuseNode),
}

pub struct DirectoryFuseNode {
    pub attributes: FileAttr,
    pub name: String,
    children_getter: Box<dyn Fn() -> Vec<u64>>,
    extra_children: Vec<u64>,
}

impl<'a> DirectoryFuseNode {
    pub fn new(
        name: &str,
        attributes: FileAttr,
        children_getter: Box<dyn Fn() -> Vec<u64>>,
    ) -> Self {
        DirectoryFuseNode {
            attributes,
            name: name.to_string(),
            children_getter,
            extra_children: Default::default(),
        }
    }

    pub fn get_children(&self) -> Vec<u64> {
        let mut children = (self.children_getter)();
        children.extend(self.extra_children.to_vec());
        return children;
    }

    pub fn get_inode_number(&self) -> u64 {
        self.attributes.ino
    }
}

// TODO: move away from frame
pub struct FrameFileFuseNode {
    pub attributes: FileAttr,
    pub frame_id: u64,
    pub directory_inode_number: u64,
}

impl FrameFileFuseNode {
    pub fn get_name(&self) -> String {
        return format!("frame-{}.jpg", self.frame_id);
    }

    pub fn get_inode_number(&self) -> u64 {
        self.attributes.ino
    }
}

pub struct FuseNodeStore<'a> {
    file_nodes: HashMap<u64, Box<FrameFileFuseNode>>,
    directory_nodes: HashMap<u64, Box<DirectoryFuseNode>>,
    // XXX: it would be more efficient to store the references but hit lifetime issues...
    current_inode_number: u64,
    phantom: PhantomData<&'a ()>,
}

impl<'a> FuseNodeStore<'a> {
    pub fn new() -> Self {
        let mut fuse_node_store = FuseNodeStore {
            file_nodes: Default::default(),
            directory_nodes: Default::default(),
            current_inode_number: ROOT_INODE_NUMBER,
            phantom: Default::default(),
        };
        fuse_node_store.insert_directory(
            DirectoryFuseNode {
                attributes: create_directory_attributes(ROOT_INODE_NUMBER),
                name: "root".to_string(),
                children_getter: Box::new(|| vec![]),
                extra_children: Default::default(),
            },
            ROOT_INODE_NUMBER,
        );
        return fuse_node_store;
    }

    pub fn get_root_directory(&self) -> &DirectoryFuseNode {
        &mut self.get_directory_node(ROOT_INODE_NUMBER).expect(&format!(
            "Expected to find root note with inode number: {}",
            ROOT_INODE_NUMBER
        ))
    }

    // TODO: required?
    pub fn insert_directory(
        &mut self,
        directory: DirectoryFuseNode,
        parent_directory_inode_number: u64,
    ) {
        let inode_number = directory.get_inode_number();
        self.directory_nodes
            .insert(inode_number, Box::new(directory));
        self.add_child_to_directory(inode_number, parent_directory_inode_number);
    }

    // TODO: split into factory?
    pub fn create_and_insert_directory(
        &mut self,
        name: &str,
        parent_directory_inode_number: u64,
    ) -> u64 {
        let inode_number = self.create_inode_number();
        let node = DirectoryFuseNode {
            attributes: create_directory_attributes(inode_number),
            name: name.to_string(),
            children_getter: Box::new(|| vec![]),
            extra_children: vec![],
        };
        self.insert_directory(node, parent_directory_inode_number);
        return inode_number;
    }

    // TODO: how many users of this function are there?
    fn add_child_to_directory(&mut self, inode_number: u64, parent_directory_inode_number: u64) {
        let parent_directory = self
            .directory_nodes
            .get_mut(&parent_directory_inode_number)
            .expect(&format!(
                "Parent directory does not exist: {}",
                parent_directory_inode_number
            ));
        parent_directory.extra_children.push(inode_number);
    }

    pub fn insert_file(
        &mut self,
        file_node: FrameFileFuseNode,
        directory_inode_number: u64,
    ) -> u64 {
        let inode_number = self.create_inode_number();
        self.file_nodes.insert(inode_number, Box::new(file_node));
        self.directory_nodes
            .get_mut(&directory_inode_number)
            .expect(&format!(
                "Could not get directory: {}",
                directory_inode_number
            ))
            .extra_children
            .push(inode_number);
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
            Some(boxed_node) => Some(&boxed_node),
            None => None,
        }
    }

    pub fn get_node(&self, inode_number: u64) -> Option<FuseNode> {
        eprintln!("Getting node {}", inode_number);
        match self.get_file_node(inode_number) {
            Some(x) => Some(FuseNode::File(x)),
            None => match self.get_directory_node(inode_number) {
                Some(x) => Some(FuseNode::Directory(x)),
                None => None,
            },
        }
    }

    pub fn lookup_node(&self, name: &str, directory_inode_number: u64) -> Option<FuseNode> {
        let directory = self
            .directory_nodes
            .get(&directory_inode_number)
            .expect(&format!(
                "Directory with inode number does not exist: {}",
                directory_inode_number
            ));

        // TODO: specialised data structure to
        for child_inode_number in directory.get_children() {
            match self.get_node(child_inode_number) {
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
            .directory_nodes
            .get(&directory_inode_number)
            .expect(&format!(
                "Non-existent directory {}",
                directory_inode_number
            ))
            .get_children()
            .iter()
            .map(|inode_number| {
                self.get_node(*inode_number).expect(&format!(
                    "Directory contains node that does not exist: {}",
                    inode_number
                ))
            })
            .collect();
    }

    pub fn create_inode_number(&mut self) -> u64 {
        self.current_inode_number += 1;
        return self.current_inode_number;
    }
}
