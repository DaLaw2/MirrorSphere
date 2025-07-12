use crate::model::error::io::IOError;
use crate::model::error::Error;
use blake2::{Blake2b512, Blake2s256};
use digest::{Digest, DynDigest, HashMarker};
use md5::Md5;
use sha2::Sha256;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn md5(path: PathBuf) -> Result<Vec<u8>, Error> {
    let hasher = Md5::new();
    file_hash(path, hasher)
}

pub fn sha3(path: PathBuf) -> Result<Vec<u8>, Error> {
    let hasher = sha3::Sha3_256::new();
    file_hash(path, hasher)
}

pub fn sha256(path: PathBuf) -> Result<Vec<u8>, Error> {
    let hasher = Sha256::new();
    file_hash(path, hasher)
}

pub fn blake2b(path: PathBuf) -> Result<Vec<u8>, Error> {
    let hasher = Blake2b512::new();
    file_hash(path, hasher)
}

pub fn blake2s(path: PathBuf) -> Result<Vec<u8>, Error> {
    let hasher = Blake2s256::new();
    file_hash(path, hasher)
}

pub fn blake3(path: PathBuf) -> Result<Vec<u8>, Error> {
    let hasher = blake3::Hasher::new();
    file_hash(path, hasher)
}

fn file_hash(path: PathBuf, mut hasher: impl HashMarker + DynDigest) -> Result<Vec<u8>, Error> {
    let mut file = File::open(&path).map_err(|err| IOError::ReadFileFailed(path.clone(), err))?;
    let mut buffer = [0; 65536];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| IOError::ReadFileFailed(path.clone(), err))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(Box::new(hasher).finalize().to_vec())
}
