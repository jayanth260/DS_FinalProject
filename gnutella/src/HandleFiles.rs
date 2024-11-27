use std::fs::{self, File, metadata};
use std::io::{self, Result};
use std::path::Path;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static SHARED_FILES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static SHARED_FILES_KB: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

pub struct PathValidator;

impl PathValidator {
    // validate and store the file paths, and also calculate the total size in kilobytes.
    pub fn validate_and_store_file_paths(file_paths_list: &str) -> Result<(usize, usize)> {
        let paths_content = fs::read_to_string(file_paths_list)?;
        let paths: Vec<&str> = paths_content.lines().collect();
        let mut validated_paths = Vec::new();
        let mut total_kb = 0;
        
        for path_str in paths {
            let path = Path::new(path_str);
            if path.exists() && path.is_file() {
                match File::open(path) {
                    Ok(_) => {
                        match metadata(path) {
                            Ok(meta) => {
                                let file_kb = (meta.len() / 1024) as usize;
                                total_kb += file_kb;

                                validated_paths.push(path_str.to_string());
                                println!("Validated file path: {} ({} KB)", path_str, file_kb);
                            },
                            Err(e) => {
                                eprintln!("Cannot get metadata for {}: {}", path_str, e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Cannot open file {}: {}", path_str, e);
                    }
                }
            } else {
                eprintln!("Invalid file path or not a file: {}", path_str);
            }
        }
        
        if let Ok(mut shared_files) = SHARED_FILES.lock() {
            *shared_files = validated_paths.clone();
        }
        
        if let Ok(mut shared_files_kb) = SHARED_FILES_KB.lock() {
            *shared_files_kb = total_kb;
        }

        Ok((validated_paths.len(), total_kb))
    }


    // get the vector of shared files.
    pub fn get_shared_files() -> Vec<String> {
        if let Ok(shared_files) = SHARED_FILES.lock() {
            shared_files.clone()
        } else {
            Vec::new()
        }
    }
    
    // get the total kilobytes of shared files.
    pub fn get_shared_files_kb() -> usize {
        if let Ok(shared_files_kb) = SHARED_FILES_KB.lock() {
            *shared_files_kb
        } else {
            0
        }
    }
    
    // check if a specific file path is in the list of shared paths.
    // pub fn is_file_shared(file_path: &str) -> bool {
    //     if let Ok(shared_files) = SHARED_FILES.lock() {
    //         shared_files.contains(&file_path.to_string())
    //     } else {
    //         false
    //     }
    // }
  

pub fn is_file_shared(file_path: &str) -> Option<(usize, u32)> {
    // Extract the filename from the full path
    let filename = file_path.split('/').last().unwrap_or(file_path);

    if let Ok(shared_files) = SHARED_FILES.lock() {
        // Find the position of the first shared file that matches the given filename
        if let Some(index) = shared_files.iter().position(|shared_file| {
            shared_file.split('/').last().unwrap_or(shared_file) == filename
        }) {
            // Get the size of the file in bytes
            let size = fs::metadata(&shared_files[index])
                .map(|metadata| metadata.len())
                .unwrap_or(0); // Use 0 if metadata fails
            Some((size.try_into().unwrap(),index.try_into().unwrap() ))
        } else {
            None
        }
    } else {
        None // Return None if unable to acquire the lock
    }
}

    
}