//! Bounds-based recursive filesystem metadata.

type DataBounds = (usize, usize); // start, size

/// A node present in the filesystem tree; variants contain metadata.
#[derive(Debug)]
pub enum Node {
    File(File),
    Dir(Dir),
}

/// The inner type of a [`Node::File`]
///
/// [`Node::File`]: enum.Node.html#File.v
#[derive(Debug)]
pub struct File {
    name: String,
    data_bounds: DataBounds,
}

/// The inner type of a [`Node::Dir`]
///
/// [`Node::Dir`]: enum.Node.html#Dir.v
#[derive(Debug)]
pub struct Dir {
    name: String,
    pub members: Vec<Box<Node>>,
}

impl Dir {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new<S: Into<String>>(name: S) -> Dir {
        Dir {
            name: name.into(),
            members: Vec::new(),
        }
    }

    pub fn add(&mut self, node: Node) {
        self.members.push(Box::new(node));
    }
}

impl File {
    pub fn new<S: Into<String>>(name: S, data_bounds: DataBounds) -> File {
        File {
            name: name.into(),
            data_bounds: data_bounds,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A filesystem. Contains a root [`Dir`].
///
/// [`Dir`]: struct.Dir.html
#[derive(Debug)]
pub struct Fs {
    pub root: Dir,
}

impl Fs {
    pub fn new(root: Dir) -> Fs {
        Fs { root: root }
    }
}

/// Dumps a tree view of a [`Dir`].
///
/// [`Dir`]: struct.Dir.html
pub fn dump_tree(dir: &Dir) {
    const INDENT: usize = 2;
    fn dump_tree(d: &Dir, level: usize) {
        println!("{}{}", " ".repeat(level * INDENT), d.name());
        for n in &d.members {
            match **n {
                Node::File(ref f) => println!("{}{}", " ".repeat((level + 1) * INDENT), f.name()),
                Node::Dir(ref d) => {
                    dump_tree(d, level + 1);
                }
            }
        }
    }

    dump_tree(dir, 0);
}
