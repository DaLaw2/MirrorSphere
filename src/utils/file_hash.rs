use digest::{Digest, HashMarker};
use md5::Md5;
use std::fs::File;
use std::io::Read;
use blake3::Hash;
use blake2::{Blake2b512, Blake2s256};
use sha2::Sha256;
use crate::utils::log_entry::io::IOEntry;

pub fn md5(file: File) -> anyhow::Result<Vec<u8>> {
    let hasher = Md5::new();
    file_hash(file, hasher)
}

pub fn sha3(file: File) -> anyhow::Result<Vec<u8>> {
    let hasher = sha3::Sha3_256::new();
    file_hash(file, hasher)
}

pub fn sha256(file: File) -> anyhow::Result<Vec<u8>> {
    let hasher = Sha256::new();
    file_hash(file, hasher)
}

pub fn blake2b(file: File) -> anyhow::Result<Vec<u8>> {
    let hasher = Blake2b512::new();
    file_hash(file, hasher)
}

pub fn blake2s(file: File) -> anyhow::Result<Vec<u8>> {
    let hasher = Blake2s256::new();
    file_hash(file, hasher)
}

pub fn blake3(file: File) -> anyhow::Result<Vec<u8>> {
    let hasher = blake3::Hasher::new();
    file_hash(file, hasher)
}

fn file_hash(mut file: File, mut hasher: impl HashMarker) -> anyhow::Result<Vec<u8>> {
    let mut buffer = [0; 65536];
    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|_| IOEntry::ReadFileFailed)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.finalize().to_vec())
}
