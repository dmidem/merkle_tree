use std::collections::HashMap;

use std::{
    fs,
    io::{self, Read, Seek},
};

use merkle_tree::{hasher, tree};

type Hasher = hasher::SdbmHasher;
type Hash = <Hasher as hasher::MerkleTreeHasher>::Hash;

pub type Tree = tree::MerkleTree<Hasher>;

fn read_file_chunk<File: Read>(file: &mut File, chunk_size: usize) -> io::Result<Vec<u8>> {
    let mut bytes_read = 0;
    let mut buffer = vec![0u8; chunk_size];

    while bytes_read < buffer.len() {
        match file.read(&mut buffer[bytes_read..])? {
            0 => break,
            n => bytes_read += n,
        }
    }

    Ok(buffer)
}

fn read_file_chunk_by_offet(
    file_path: &std::path::PathBuf,
    offset: u64,
    chunk_size: usize,
) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(file_path)?;
    file.seek(io::SeekFrom::Start(offset))?;
    read_file_chunk(&mut file, chunk_size)
}

fn make_merkle_tree_for_file(
    file_path: &std::path::PathBuf,
    chunk_size: usize,
) -> Result<(Tree, u64), String> {
    let mut file =
        fs::File::open(file_path).map_err(|_| format!("can not open file {:?}", file_path))?;

    let file_size = file
        .metadata()
        .map_err(|_| format!("can not get size of file {:?}", file_path))?
        .len();

    let chunks_number: usize = ((file_size + chunk_size as u64 - 1) / chunk_size as u64)
        .try_into()
        .map_err(|_| format!("can not calculate chunks number for file {:?}", file_path))?;

    Ok((
        Tree::try_from_data_items(
            (0..chunks_number).map(|_| read_file_chunk(&mut file, chunk_size)),
        )
        .map_err(|_| format!("can not read file {:?}", file_path))?,
        file_size,
    ))
}

#[derive(Debug)]
struct FileHash {
    path: std::path::PathBuf,
    size: u64,
    tree: Tree,
}

#[derive(Debug)]
pub struct FileInfo<'a> {
    pub name: &'a std::ffi::OsStr,
    pub size: u64,
    pub root_hash: &'a Hash,
    pub chunk_size: usize,
}

#[derive(Debug)]
pub struct FileServer {
    chunk_size: usize,
    files: HashMap<Hash, FileHash>,
}

impl FileServer {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            files: HashMap::new(),
        }
    }

    pub fn from_dir<P: AsRef<std::path::Path>, Ext: AsRef<str>>(
        dir_path: P,
        allowed_extensions: &[Ext],
        chunk_size: usize,
    ) -> Result<Self, String> {
        let mut server = FileServer::new(chunk_size);
        server.hash_files_from_dir(dir_path, allowed_extensions)?;
        Ok(server)
    }

    pub fn hash_file(&mut self, file_path: std::path::PathBuf) -> Result<(), String> {
        let (tree, file_size) = make_merkle_tree_for_file(&file_path, self.chunk_size)?;
        let hash = tree
            .get_root()
            .ok_or_else(|| format!("empty file: {:?}", file_path))?;

        if self
            .files
            .insert(
                hash,
                FileHash {
                    path: file_path.clone(),
                    size: file_size,
                    tree,
                },
            )
            .is_some()
        {
            return Err(format!(
                "hash ({:?}) collision for file {:?}",
                hash, file_path
            ));
        }

        Ok(())
    }

    pub fn hash_files_from_dir<P: AsRef<std::path::Path>, Ext: AsRef<str>>(
        &mut self,
        dir_path: P,
        allowed_extensions: &[Ext],
    ) -> Result<(), String> {
        for entry in fs::read_dir(dir_path.as_ref())
            .map_err(|_| format!("can not read directory {:?}", dir_path.as_ref()))?
            .filter_map(|entry| {
                let entry = entry.ok()?;

                if !entry.file_type().ok()?.is_file() {
                    None
                } else if allowed_extensions.is_empty() {
                    Some(entry)
                } else {
                    let path = entry.path();
                    let ext = path.extension()?;

                    allowed_extensions
                        .iter()
                        .any(|allowed_extension| ext == allowed_extension.as_ref())
                        .then_some(entry)
                }
            })
        {
            self.hash_file(entry.path())?;
        }

        Ok(())
    }

    pub fn list_files(&self) -> impl Iterator<Item = FileInfo> {
        self.files.iter().filter_map(|(root_hash, file_item)| {
            file_item.path.file_name().map(|file_name| FileInfo {
                name: file_name,
                size: file_item.size,
                root_hash,
                chunk_size: self.chunk_size,
            })
        })
    }

    pub fn get_file_chunk(
        &self,
        file_hash: Hash,
        chunk_index: usize,
    ) -> Option<(tree::Proof<Hasher>, Vec<u8>)> {
        let file_info = self.files.get(&file_hash)?;
        match read_file_chunk_by_offet(
            &file_info.path,
            chunk_index as u64 * self.chunk_size as u64,
            self.chunk_size,
        ) {
            Ok(data) if !data.is_empty() => file_info
                .tree
                .get_proof(chunk_index)
                .map(|proof| (proof, data)),
            _ => None,
        }
    }
}
