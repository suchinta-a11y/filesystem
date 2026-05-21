use std::collections::BTreeMap;
use std::fmt;

pub enum Node {
    File(String),
    Directory(BTreeMap<String, Node>),
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Node>")
    }
}

pub struct Filesystem {
    pub root: BTreeMap<String, Node>,
}

impl Filesystem {
    pub fn new() -> Self {
        Self {
            root: BTreeMap::new(),
        }
    }
}

pub enum Path {
    File(String),
    Directory(String, Box<Path>),
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Path::File(name) => write!(f, "{}", name),
            Path::Directory(name, rest) => write!(f, "{}/{}", name, rest),
        }
    }
}

impl FromIterator<String> for Path {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let parts: Vec<String> = iter.into_iter().collect();
        fn build(parts: &[String]) -> Path {
            if parts.len() == 1 {
                Path::File(parts[0].clone())
            } else {
                Path::Directory(parts[0].clone(), Box::new(build(&parts[1..])))
            }
        }
        build(&parts)
    }
}

#[derive(Debug)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotAFile,
    NotADirectory,
}

#[derive(Debug)]
pub struct FileHandle<'a>(&'a mut String);

// I struggled here with lifetimes — I kept trying to store a copy of the string
// but the write function needs to actually modify the data inside the filesystem,
// so it has to be a mutable reference with a lifetime tied to the filesystem.
impl<'a> FileHandle<'a> {
    pub fn read(&self) -> &str {
        &self.0
    }

    pub fn write(&mut self, data: String) {
        *self.0 = data;
    }
}

pub struct DirectoryHandle<'a>(pub &'a mut BTreeMap<String, Node>);

impl fmt::Debug for DirectoryHandle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<DirectoryHandle>")
    }
}

impl<'a> DirectoryHandle<'a> {
    pub fn create(&mut self, name: String, contents: String) -> Result<FileHandle<'_>, FsError> {
        if self.0.contains_key(&name) {
            return Err(FsError::AlreadyExists);
        }
        self.0.insert(name.clone(), Node::File(contents));
        match self.0.get_mut(&name).unwrap() {
            Node::File(s) => Ok(FileHandle(s)),
            _ => unreachable!(),
        }
    }

    pub fn create_dir(&mut self, name: String) -> Result<DirectoryHandle<'_>, FsError> {
        if self.0.contains_key(&name) {
            return Err(FsError::AlreadyExists);
        }
        self.0.insert(name.clone(), Node::Directory(BTreeMap::new()));
        match self.0.get_mut(&name).unwrap() {
            Node::Directory(map) => Ok(DirectoryHandle(map)),
            _ => unreachable!(),
        }
    }
}

pub enum NodeHandle<'a> {
    File(FileHandle<'a>),
    Dir(DirectoryHandle<'a>),
}

impl fmt::Debug for NodeHandle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<NodeHandle>")
    }
}

impl<'a> NodeHandle<'a> {
    pub fn file_handle(self) -> Result<FileHandle<'a>, FsError> {
        match self {
            NodeHandle::File(fh) => Ok(fh),
            NodeHandle::Dir(_) => Err(FsError::NotAFile),
        }
    }

    pub fn dir_handle(self) -> Result<DirectoryHandle<'a>, FsError> {
        match self {
            NodeHandle::Dir(dh) => Ok(dh),
            NodeHandle::File(_) => Err(FsError::NotADirectory),
        }
    }
}

pub trait Find {
    fn find(&mut self, path: &Path) -> Result<NodeHandle<'_>, FsError>;
}

// I struggled here figuring out how to share the recursive find logic between
// Filesystem and DirectoryHandle without duplicating code. I ended up pulling
// it into a separate helper function that both implementations call.
fn find_in<'a>(
    map: &'a mut BTreeMap<String, Node>,
    path: &Path,
) -> Result<NodeHandle<'a>, FsError> {
    match path {
        Path::File(name) => match map.get_mut(name) {
            None => Err(FsError::NotFound),
            Some(Node::File(s)) => Ok(NodeHandle::File(FileHandle(s))),
            Some(Node::Directory(m)) => Ok(NodeHandle::Dir(DirectoryHandle(m))),
        },
        Path::Directory(name, rest) => match map.get_mut(name) {
            None => Err(FsError::NotFound),
            Some(Node::File(_)) => Err(FsError::NotADirectory),
            Some(Node::Directory(m)) => find_in(m, rest),
        },
    }
}

impl Find for Filesystem {
    fn find(&mut self, path: &Path) -> Result<NodeHandle<'_>, FsError> {
        find_in(&mut self.root, path)
    }
}

impl<'a> Find for DirectoryHandle<'a> {
    fn find(&mut self, path: &Path) -> Result<NodeHandle<'_>, FsError> {
        find_in(self.0, path)
    }
}

impl fmt::Display for Filesystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "/")?;
        fmt_dir(f, &self.root, 0)
    }
}

fn fmt_dir(
    f: &mut fmt::Formatter<'_>,
    map: &BTreeMap<String, Node>,
    level: usize,
) -> fmt::Result {
    let indent = "   ".repeat(level);
    for (name, node) in map {
        match node {
            Node::File(_) => writeln!(f, "{}|- {}", indent, name)?,
            Node::Directory(children) => {
                writeln!(f, "{}|- {}/", indent, name)?;
                fmt_dir(f, children, level + 1)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;