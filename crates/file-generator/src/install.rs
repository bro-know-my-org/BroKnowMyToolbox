//! Install a completed temporary file without replacing an existing directory entry.
//! Names are single path components relative to the already-confined parent handle.

use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path};

use cap_std::fs::{Dir, File};

pub(crate) fn create(
    parent: &Dir,
    temporary: &File,
    source: &str,
    target: &OsStr,
) -> io::Result<()> {
    for name in [OsStr::new(source), target] {
        let mut components = Path::new(name).components();
        if !matches!(components.next(), Some(Component::Normal(part)) if part == name)
            || components.next().is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a file name",
            ));
        }
    }
    create_impl(parent, temporary, source, target)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn create_impl(parent: &Dir, _temporary: &File, source: &str, target: &OsStr) -> io::Result<()> {
    match rustix::fs::renameat_with(
        parent,
        source,
        parent,
        target,
        rustix::fs::RenameFlags::NOREPLACE,
    ) {
        Ok(()) => Ok(()),
        Err(rustix::io::Errno::NOSYS | rustix::io::Errno::INVAL | rustix::io::Errno::OPNOTSUPP) => {
            create_with_hard_link(parent, source, target)
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(windows)]
fn create_impl(parent: &Dir, temporary: &File, _source: &str, target: &OsStr) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_RENAME_INFO, FileRenameInfo, SetFileInformationByHandle,
    };

    let name = target.encode_wide().collect::<Vec<_>>();
    let name_bytes = name
        .len()
        .checked_mul(2)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "file name is too long"))?;
    let length = std::mem::offset_of!(FILE_RENAME_INFO, FileName)
        .checked_add(name_bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "file name is too long"))?
        .max(std::mem::size_of::<FILE_RENAME_INFO>());
    let length_u32 = u32::try_from(length)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "file name is too long"))?;
    // u64 storage gives FILE_RENAME_INFO its required alignment, including on x64.
    let mut storage = vec![0u64; length.div_ceil(8)];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    // SAFETY: storage is aligned, initialized, and large enough for the header and
    // UTF-16 tail. Both handles remain alive through this synchronous call. The
    // zeroed ReplaceIfExists field deliberately forbids replacing any target.
    let result = unsafe {
        (*info).RootDirectory = parent.as_raw_handle();
        (*info).FileNameLength = name_bytes as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
            name.len(),
        );
        SetFileInformationByHandle(
            temporary.as_raw_handle(),
            FileRenameInfo,
            info.cast(),
            length_u32,
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn create_impl(parent: &Dir, _temporary: &File, source: &str, target: &OsStr) -> io::Result<()> {
    create_with_hard_link(parent, source, target)
}

#[cfg(not(windows))]
fn create_with_hard_link(parent: &Dir, source: &str, target: &OsStr) -> io::Result<()> {
    parent.hard_link(source, parent, target)?;
    let _ = parent.remove_file(source);
    Ok(())
}
