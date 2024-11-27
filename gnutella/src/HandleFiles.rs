use std::fs::{self, File, metadata};
use std::io::{self, Result};
use std::path::Path;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static SHARED_FILES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static SHARED_FILES_KB: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

#[derive(Clone, Debug)]
pub struct DownloadedFileMetadata {
    pub file_path: String,
    pub original_host: String,
    pub original_port: String,
    pub original_index: u32,
    pub file_size: u32,
    pub original_servent_id: String,
}

pub static DOWNLOADED_FILES: Lazy<Mutex<Vec<DownloadedFileMetadata>>> = 
    Lazy::new(|| Mutex::new(Vec::new()));

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
    pub fn is_file_shared(file_path: &str) -> Vec<(usize, u32)> {
        let filename = file_path.split('/').last().unwrap_or(file_path);
        let mut results = Vec::new();

        // Check original shared files
        if let Ok(shared_files) = SHARED_FILES.lock() {
            results.extend(
                shared_files.iter()
                    .enumerate()
                    .filter_map(|(index, shared_file)| {
                        let shared_filename = shared_file.split('/').last().unwrap_or(shared_file);
                        if shared_filename == filename {
                            let size = fs::metadata(shared_file)
                                .map(|metadata| metadata.len())
                                .unwrap_or(0);
                            Some((size.try_into().unwrap(), index.try_into().unwrap()))
                        } else {
                            None
                        }
                    })
            );
        }

        // Check downloaded files
        if let Ok(downloaded_files) = DOWNLOADED_FILES.lock() {
            results.extend(
                downloaded_files.iter()
                    .enumerate()
                    .filter_map(|(index, metadata)| {
                        let downloaded_filename = metadata.file_path.split('/').last().unwrap_or(&metadata.file_path);
                        if downloaded_filename == filename {
                            Some((metadata.file_size as usize, (index + 1000) as u32)) // Offset index to avoid conflicts
                        } else {
                            None
                        }
                    })
            );
        }

        results
    }

    pub fn add_downloaded_file(
        file_path: String,
        host: String,
        port: String,
        index: u32,
        size: u32,
        servent_id: String,
    ) -> io::Result<()> {
        let metadata = DownloadedFileMetadata {
            file_path,
            original_host: host,
            original_port: port,
            original_index: index,
            file_size: size,
            original_servent_id: servent_id,
        };

        if let Ok(mut downloaded_files) = DOWNLOADED_FILES.lock() {
            downloaded_files.push(metadata);
            
            // Update the total KB count
            if let Ok(mut shared_files_kb) = SHARED_FILES_KB.lock() {
                *shared_files_kb += (size / 1024) as usize;
            }
        }

        Ok(())
    }

    // New method to get file metadata by index
    pub fn get_file_metadata(index: u32) -> Option<DownloadedFileMetadata> {
        if let Ok(downloaded_files) = DOWNLOADED_FILES.lock() {
            if index >= 1000 { // Check if it's a downloaded file
                downloaded_files.get(index as usize - 1000).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
}