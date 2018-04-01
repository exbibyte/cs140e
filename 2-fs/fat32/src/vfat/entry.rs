use traits;
use vfat::{File, Dir, Metadata, Cluster};

// TODO: You may need to change this definition.
#[derive(Debug)]
pub enum Entry {
    File(File),
    Dir(Dir)
}

// TODO: Implement any useful helper methods on `Entry`.

// FIXME: Implement `traits::Entry` for `Entry`.

/// Trait implemented by directory entries in a file system.
///
/// An entry is either a `File` or a `Directory` and is associated with both
/// `Metadata` and a name.
impl traits::Entry for Entry {
    type File = File;
    type Dir = Dir;
    type Metadata = Metadata;

    /// The name of the file or directory corresponding to this entry.
    fn name(&self) -> &str {
        match self {
            &Entry::File(ref x) => {
                if x.lfn.len() > 0 {
                    x.lfn.as_str()
                } else {
                    x.short_file_name.as_str()
                }
            }
            &Entry::Dir(ref x) => {
                if x.lfn.len() > 0 {
                    x.lfn.as_str()
                } else {
                    x.short_file_name.as_str()
                }
            },
        }
    }

    /// The metadata associated with the entry.
    fn metadata(&self) -> &Self::Metadata {
        match self {
            &Entry::File(ref x) => &x.meta,
            &Entry::Dir(ref x) => &x.meta
        }
    }

    /// If `self` is a file, returns `Some` of a reference to the file.
    /// Otherwise returns `None`.
    fn as_file(&self) -> Option<&Self::File> {
        match self {
            &Entry::File(ref x) => Some(x),
            _ => None,
        }
    }

    /// If `self` is a directory, returns `Some` of a reference to the
    /// directory. Otherwise returns `None`.
    fn as_dir(&self) -> Option<&Self::Dir> {
        match self {
            &Entry::Dir(ref x) => Some(x),
            _ => None,
        }
    }

    /// If `self` is a file, returns `Some` of the file. Otherwise returns
    /// `None`.
    fn into_file(self) -> Option<Self::File> {
        match self {
            Entry::File(x) => Some(x),
            _ => None,
        }        
    }

    /// If `self` is a directory, returns `Some` of the directory. Otherwise
    /// returns `None`.
    fn into_dir(self) -> Option<Self::Dir> {
        match self {
            Entry::Dir(x) => Some(x),
            _ => None,
        }
    }
}
