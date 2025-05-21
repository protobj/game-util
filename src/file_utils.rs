use std::fs::{self};
use std::io::{self};
use std::path::Path;

pub fn check_dir_path(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)?;
    } else if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "路径存在但不是目录",
        ));
    }
    Ok(())
}

pub fn check_file_exist(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

pub fn copy_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    // 确保目标目录存在
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::copy(src, dst)?;
    Ok(())
}

pub fn copy_dir_files_no_subdir(
    src_dir: impl AsRef<Path>,
    dst_dir: impl AsRef<Path>,
    skip_filter: impl Fn(&fs::DirEntry) -> bool,
) -> io::Result<()> {
    let src_dir = src_dir.as_ref();
    let dst_dir = dst_dir.as_ref();
    
    // 确保目标目录存在
    fs::create_dir_all(dst_dir)?;
    
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && !skip_filter(&entry) {
            let file_name = path.file_name().unwrap();
            let dst_path = dst_dir.join(file_name);
            fs::copy(&path, dst_path)?;
        }
    }
    
    Ok(())
}