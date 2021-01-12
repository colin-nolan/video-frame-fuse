use crate::fuse_fs::models::file::FileInformation;
use fuse::{FileAttr, FileType};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::SystemTime;
use users::{get_current_gid, get_current_uid};

pub const ROOT_INODE_NUMBER: u64 = 1;

// TODO: sort
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

pub fn create_file_attributes(
    inode_number: u64,
    size: u64,
    executable: bool,
    writable: bool,
) -> FileAttr {
    let mut permissions = 0o440;
    if writable {
        permissions += 0o220;
    }
    if executable {
        permissions += 0o110;
    }
    return FileAttr {
        ino: inode_number,
        size,
        blocks: 1,
        atime: SystemTime::UNIX_EPOCH,
        mtime: SystemTime::UNIX_EPOCH,
        ctime: SystemTime::UNIX_EPOCH,
        crtime: SystemTime::UNIX_EPOCH,
        kind: FileType::RegularFile,
        perm: permissions,
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
    File(&'a FileFuseNode),
}

pub struct DirectoryFuseNode {
    pub attributes: FileAttr,
    pub name: String,
    file_information_generator: Option<Box<dyn Fn(u64) -> Vec<FileInformation>>>,
    children_to_generate_from_file_information: bool,
    children_inode_numbers: Vec<u64>,
}

impl DirectoryFuseNode {
    pub fn new(
        name: &str,
        attributes: FileAttr,
        children_generator: Box<dyn Fn(u64) -> Vec<FileInformation>>,
    ) -> Self {
        DirectoryFuseNode {
            attributes,
            name: name.to_string(),
            file_information_generator: Some(children_generator),
            children_inode_numbers: Default::default(),
            children_to_generate_from_file_information: true,
        }
    }

    pub fn get_inode_number(&self) -> u64 {
        self.attributes.ino
    }
}

pub struct FileFuseNode {
    pub information: FileInformation,
    pub directory_inode_number: u64,
    inode_number: u64,
}

impl FileFuseNode {
    pub fn get_inode_number(&self) -> u64 {
        self.inode_number
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.information.get_data()
    }

    pub fn get_attributes(&self) -> FileAttr {
        create_file_attributes(
            self.get_inode_number(),
            self.get_data().len() as u64,
            self.information.executable,
            self.information.writable,
        )
    }
}

pub struct FuseNodeStore<'a> {
    file_nodes: HashMap<u64, Box<FileFuseNode>>,
    directory_nodes: HashMap<u64, Box<DirectoryFuseNode>>,
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
                file_information_generator: None,
                children_inode_numbers: Default::default(),
                children_to_generate_from_file_information: false,
            },
            ROOT_INODE_NUMBER,
        );
        return fuse_node_store;
    }

    pub fn get_root_directory(&self) -> &DirectoryFuseNode {
        &self.get_directory_node(ROOT_INODE_NUMBER).expect(&format!(
            "Expected to find root note with inode number: {}",
            ROOT_INODE_NUMBER
        ))
    }

    pub fn insert_directory(
        &mut self,
        directory: DirectoryFuseNode,
        parent_directory_inode_number: u64,
    ) {
        let inode_number = directory.get_inode_number();
        self.directory_nodes
            .insert(inode_number, Box::new(directory));

        let parent_directory = self
            .directory_nodes
            .get_mut(&parent_directory_inode_number)
            .expect(&format!(
                "Parent directory does not exist: {}",
                parent_directory_inode_number
            ));
        parent_directory.children_inode_numbers.push(inode_number);
    }

    pub fn create_and_insert_directory(
        &mut self,
        name: &str,
        parent_directory_inode_number: u64,
    ) -> u64 {
        let inode_number = self.create_inode_number();
        let node = DirectoryFuseNode {
            attributes: create_directory_attributes(inode_number),
            name: name.to_string(),
            file_information_generator: None,
            children_inode_numbers: Default::default(),
            children_to_generate_from_file_information: false,
        };
        self.insert_directory(node, parent_directory_inode_number);
        return inode_number;
    }

    pub fn create_and_insert_file(
        &mut self,
        file_information: FileInformation,
        directory_inode_number: u64,
    ) -> u64 {
        let inode_number = self.create_inode_number();
        let file_node = FileFuseNode {
            information: file_information,
            directory_inode_number,
            inode_number,
        };

        self.file_nodes.insert(inode_number, Box::new(file_node));
        self.directory_nodes
            .get_mut(&directory_inode_number)
            .expect(&format!(
                "Could not get directory: {}",
                directory_inode_number
            ))
            .children_inode_numbers
            .push(inode_number);
        return inode_number;
    }

    pub fn get_file_node(&self, inode_number: u64) -> Option<&FileFuseNode> {
        match self.file_nodes.get(&inode_number) {
            Some(boxed_node) => Some(boxed_node.as_ref()),
            None => None,
        }
    }

    pub fn get_file_node_mut(&mut self, inode_number: u64) -> Option<&mut FileFuseNode> {
        match self.file_nodes.get_mut(&inode_number) {
            Some(boxed_node) => Some(boxed_node.as_mut()),
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
        match self.get_file_node(inode_number) {
            Some(x) => Some(FuseNode::File(x)),
            None => match self.get_directory_node(inode_number) {
                Some(x) => Some(FuseNode::Directory(x)),
                None => None,
            },
        }
    }

    pub fn lookup_node(&mut self, name: &str, directory_inode_number: u64) -> Option<FuseNode> {
        // TODO: specialised data structure to optimise
        for child_node in self.get_nodes_in_directory(directory_inode_number) {
            let node_name = match child_node {
                FuseNode::Directory(x) => x.name.as_str(),
                FuseNode::File(x) => x.information.name.as_str(),
            };
            if node_name == name {
                return Some(child_node);
            }
        }
        return None;
    }

    pub fn get_nodes_in_directory(&mut self, directory_inode_number: u64) -> Vec<FuseNode> {
        let non_existent_director_error =
            &format!("Non-existent directory: {}", directory_inode_number);

        let directory = self
            .directory_nodes
            .get(&directory_inode_number)
            .expect(non_existent_director_error);

        if directory.children_to_generate_from_file_information {
            for file_information in directory.file_information_generator.as_ref().expect(
                "Expected children generator because flag indicated that there were children \
                    to generate",
            )(directory.get_inode_number())
            {
                self.create_and_insert_file(file_information, directory_inode_number);
            }
        }

        self.directory_nodes
            .get_mut(&directory_inode_number)
            .expect(non_existent_director_error)
            .children_to_generate_from_file_information = false;

        let directory = self
            .directory_nodes
            .get(&directory_inode_number)
            .expect(non_existent_director_error);

        let mut children = vec![];
        for child_inode_number in &directory.children_inode_numbers {
            let child_node = self
                .get_node(*child_inode_number)
                .expect(non_existent_director_error);
            children.push(child_node);
        }

        return children;
    }

    pub fn create_inode_number(&mut self) -> u64 {
        self.current_inode_number += 1;
        return self.current_inode_number;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref FILE_ATTR_1: FileAttr = FileAttr {
            ino: 42,
            size: 0,
            blocks: 0,
            atime: SystemTime::UNIX_EPOCH,
            mtime: SystemTime::UNIX_EPOCH,
            ctime: SystemTime::UNIX_EPOCH,
            crtime: SystemTime::UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0
        };
    }

    #[test]
    fn directory_fuse_node() {
        let directory_fuse_node =
            DirectoryFuseNode::new("test", FILE_ATTR_1.clone(), Box::new(|_| vec![]));
        assert_eq!(directory_fuse_node.get_inode_number(), FILE_ATTR_1.ino)
    }

    #[test]
    fn file_fuse_node() {
        let inode_number = 43;
        let data = "testing".as_bytes().to_vec();

        let file_fuse_node = FileFuseNode {
            information: FileInformation::new_with_data(
                "test",
                data.clone(),
                false,
                false,
                false,
                None,
            ),
            directory_inode_number: 0,
            inode_number: inode_number,
        };
        assert_eq!(file_fuse_node.get_inode_number(), inode_number);
        assert_eq!(file_fuse_node.get_data(), data);
        let attributes = file_fuse_node.get_attributes();
        assert_eq!(attributes.ino, inode_number);
        assert_eq!(attributes.kind, FileType::RegularFile);
        assert_eq!(attributes.size, data.len() as u64);
    }

    #[test]
    fn node_store_get_root_directory() {
        let node_store = FuseNodeStore::new();
        let root_directory = node_store.get_root_directory();

        node_store
            .directory_nodes
            .get(&root_directory.get_inode_number())
            .unwrap();
        assert_eq!(root_directory.get_inode_number(), ROOT_INODE_NUMBER);
        assert_eq!(
            node_store
                .get_directory_node(root_directory.get_inode_number())
                .unwrap()
                .get_inode_number(),
            root_directory.get_inode_number()
        );
    }

    #[test]
    fn node_store_insert_directory() {
        let mut node_store = FuseNodeStore::new();
        let name = "test123";
        let directory = DirectoryFuseNode::new(&name, FILE_ATTR_1.clone(), Box::new(|_| vec![]));
        let inode_number = directory.get_inode_number();

        node_store.insert_directory(
            directory,
            node_store.get_root_directory().get_inode_number(),
        );

        let retrieved_directory = node_store.get_directory_node(inode_number).unwrap();
        assert_eq!(&retrieved_directory.name, name);
        assert_eq!(retrieved_directory.get_inode_number(), inode_number);
    }

    #[test]
    fn node_store_create_and_insert_directory() {
        let mut node_store = FuseNodeStore::new();
        let name = "test123";
        let inode_number = node_store
            .create_and_insert_directory(&name, node_store.get_root_directory().get_inode_number());

        let retrieved_directory = node_store.get_directory_node(inode_number).unwrap();
        assert_eq!(retrieved_directory.name, name);
    }

    #[test]
    fn node_store_create_and_insert_file() {
        let mut node_store = FuseNodeStore::new();
        let data_fetcher = Box::new(|| "data".as_bytes().to_vec());
        let name = "test123";
        let file_information = FileInformation::new(&name, data_fetcher.clone(), false, false);
        let inode_number = node_store.create_and_insert_file(
            file_information,
            node_store.get_root_directory().get_inode_number(),
        );

        let retrieved_file = node_store.get_file_node(inode_number).unwrap();
        assert_eq!(retrieved_file.get_inode_number(), inode_number);
        assert_eq!(retrieved_file.information.name, name);
        assert_eq!(retrieved_file.get_data(), data_fetcher());
        assert_eq!(
            retrieved_file.get_attributes().size as usize,
            data_fetcher().len()
        );
    }

    #[test]
    fn node_store_get_file_node_not_exist() {
        let node_store = FuseNodeStore::new();
        assert!(node_store.get_node(12345).is_none());
    }

    #[test]
    fn node_store_get_directory_node_not_exist() {
        let node_store = FuseNodeStore::new();
        assert!(node_store.get_directory_node(12345).is_none());
    }

    #[test]
    fn node_store_get_node_not_exist() {
        let node_store = FuseNodeStore::new();
        assert!(node_store.get_node(12345).is_none());
    }

    #[test]
    fn node_store_get_node_directory() {
        let mut node_store = FuseNodeStore::new();
        let inode_number = node_store
            .create_and_insert_directory("", node_store.get_root_directory().get_inode_number());
        // TODO
        assert_eq!(
            node_store.get_node(inode_number).unwrap(),
            FuseNode::Directory
        )
    }

    // TODO: continue testing
}
