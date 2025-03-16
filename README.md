# MirrorSphere

A high-performance, cross-platform graphical backup tool developed in Rust.

## Features

- User-friendly graphical interface
- High-performance parallel backup algorithm
- Cross-platform support (Windows, Linux)
- Incremental backup capabilities
- File system change tracking (Windows NTFS)
- Windows NTFS ACL support
- Preservation of extended file attributes
- Real-time backup progress display

## Installation

### Download Installer

Check the [Releases](https://github.com/DaLaw2/mirrorsphere/releases) page to download the latest version of the installer.

### Build from Source

```bash
git clone https://github.com/DaLaw2/mirrorsphere.git
cd mirrorsphere
cargo build --release
```

## Usage

1. Launch the MirrorSphere application
2. In the main interface, click "New Backup Task"
3. Select source and destination directories
4. Configure backup options (thread count, enable incremental backup, etc.)
5. Click "Start Backup" button to begin the backup process

## Supported Platforms

- **Windows**: Full support, including NTFS-specific features
- **Linux**: Full support for basic backup functionality

## Technical Details

### Parallel Backup Algorithm

MirrorSphere uses a high-performance parallel processing algorithm that maximizes performance by concurrently traversing directory structures and copying files, especially suitable for large file systems and multi-core processor environments.

### Windows-Specific Features

- **USN Journal Tracking**: Utilize NTFS USN journals to quickly identify file system changes
- **ACL Support**: Preserve and restore Windows NTFS access control lists
- **Extended File Attributes**: Preserve additional file metadata

## Roadmap

- [ ] Implement file compression
- [ ] Encrypted backup options
- [ ] Remote backup (SFTP, S3, etc.)
- [ ] Backup scheduling
- [ ] macOS support
