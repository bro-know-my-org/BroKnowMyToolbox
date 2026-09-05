//! Observe an overwrite target through one confined, no-follow file handle.
//! This detects stale plans; it is not an atomic filesystem compare-and-replace.

use std::io::{self, Read};
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, Metadata, OpenOptions, Permissions};
use sha2::{Digest, Sha256};

pub(crate) struct Target {
    pub revision: String,
    pub permissions: Permissions,
}

pub(crate) fn observe(parent: &Dir, path: &Path) -> io::Result<Target> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let mut file = parent.open_with(path, &options)?;
    let before = file.metadata()?;
    if !before.is_file() {
        return Err(changed("overwrite target must be a regular file"));
    }
    #[cfg(windows)]
    {
        use cap_std::fs::MetadataExt;
        if before.file_attributes() & 0x400 != 0 {
            return Err(changed("overwrite target must not be a reparse point"));
        }
    }
    let stamp = metadata_stamp(&before)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let limit = before
        .len()
        .checked_add(1)
        .ok_or_else(|| changed("overwrite target is too large to inspect"))?;
    let mut bounded = (&mut file).take(limit);
    let mut bytes = 0u64;
    loop {
        let count = match bounded.read(&mut buffer) {
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if count == 0 {
            break;
        }
        bytes += count as u64;
        digest.update(&buffer[..count]);
    }
    if bytes != before.len() || metadata_stamp(&file.metadata()?)? != stamp {
        return Err(changed("overwrite target changed while being inspected"));
    }
    Ok(Target {
        revision: format!("v2:{stamp}:{:x}", digest.finalize()),
        permissions: before.permissions(),
    })
}

fn changed(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn metadata_stamp(metadata: &Metadata) -> io::Result<String> {
    let modified = metadata.modified()?.into_std();
    #[cfg(unix)]
    let identity = {
        use cap_std::fs::MetadataExt;
        format!(
            "{}:{}:{}:{}:{}",
            metadata.dev(),
            metadata.ino(),
            metadata.ctime(),
            metadata.ctime_nsec(),
            metadata.mode()
        )
    };
    #[cfg(windows)]
    let identity = {
        use cap_fs_ext::MetadataExt as IdentityExt;
        use cap_std::fs::MetadataExt;
        format!(
            "{}:{}:{}:{}",
            metadata.dev(),
            metadata.ino(),
            metadata.creation_time(),
            metadata.file_attributes()
        )
    };
    #[cfg(not(any(unix, windows)))]
    let identity = format!(
        "{:?}:{}",
        metadata.created().ok(),
        metadata.permissions().readonly()
    );
    Ok(format!("{}:{modified:?}:{identity}", metadata.len()))
}
