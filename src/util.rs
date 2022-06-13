// Utility functions.
// (C) Ryan Jeffrey <ryan@ryanmj.xyz>, 2022
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::fs::{Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

// Public struct definitions.

/// File object. Wrapper around fs::File but caches more data.
pub struct File {
    /// Underlying file object.
    file: fs::File,
    /// Path to the object.
    path: String,
}

// Public function definitions.

/// Return a file's contents as a byte vector on success and a string on error.
/// # Arguments
/// * `path` - Path to the file.
pub fn read_file_path(path: &String) -> Result<Vec<u8>, String> {
    File::open(&path)?.read_file()
}

/// Return two paths concatenated together on success and a string on error.
/// # Arguments
/// * `base` - Base path.
/// * `rest` - Rest of the path.
pub fn concat_paths(base: &String, rest: &String) -> Result<String, String> {
    let p = Path::new(&base).join(&rest);
    match p.to_str() {
        None => Err(format!("{} and {} joined is not valid utf8", base, rest)),
        Some(s) => Ok(s.to_string()),
    }
}

/// Get the current unix timestamp on success and a string on error.
pub fn get_unix_timestamp() -> Result<Duration, String> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(r) => Ok(r),
        Err(e) => Err(format!("Could not format unix timestamp: {}", e)),
    }
}

/// Convert a `Result<OsString, String>` to a `Result<String, String>`.
/// # Arguments
/// * `r` - Result to convert.
pub fn os_string_result_to_strings(r: Result<String, OsString>) -> Result<String, String> {
    match r {
        Err(e) => Err(match e.into_string() {
            Ok(s) => s,
            Err(ee) => "Could not coerce OS string into utf8 string".to_string(),
        }),
        Ok(rr) => Ok(rr.to_string()),
    }
}

/// Get the user's home directory. If that fails, get the current directory.
/// If that fails, return an empty string.
pub fn get_home_nofail() -> String {
    match env::var("HOME") {
        Ok(v) => format!("{}", v),
        Err(e) => {
            eprintln!("$HOME is not set. Defaulting to current directory.");
            format!(
                "{}",
                match env::current_dir() {
                    Ok(r) => match os_string_result_to_strings(r.into_os_string().into_string()) {
                        Ok(rr) => rr,
                        Err(ee) => {
                            eprintln!("Could not get current dir as utf8 string. Defaulting to nothing for $HOME: {}", e);
                            String::new()
                        }
                    },
                    Err(e) => format!("{}", e),
                }
            )
        }
    }
}

// Struct impls.

impl File {
    /// Open a file from a path. Return File on success and a string on error.
    /// # Arguments
    /// * `path` - Path to file.
    pub fn open(path: &String) -> Result<Self, String> {
        match fs::File::open(&path) {
            Ok(r) => Ok(Self {
                file: r,
                path: format!("{}", path),
            }),
            Err(e) => Err(format!("Could not open file {}: {}", path, e)),
        }
    }

    /// Open a file from a path with options. Return File on success and a string on error.
    /// # Arguments
    /// * `path` - Path to file.
    /// * `ops` - File open options.
    pub fn open_ops(path: &String, ops: &OpenOptions) -> Result<Self, String> {
        match ops.open(&path) {
            Ok(r) => Ok(Self {
                file: r,
                path: format!("{}", path),
            }),
            Err(e) => Err(format!("Could not open file {}: {}", path, e)),
        }
    }

    /// Read `self`'s contents into a byte vector, reading as many bytes as
    /// necessary to fill that vector. Return void on success and
    /// a string on error.
    /// # Arguments
    /// * `buf` - Byte vector to read `self` into.
    pub fn read_into_vec(&mut self, buf: &mut Vec<u8>) -> Result<(), String> {
        match self.file.read_exact(&mut buf[..]) {
            Ok(r) => Ok(()),
            Err(e) => Err(format!("Failed to read file {}, {}", self.path, e)),
        }
    }

    /// Read `self`'s contents into a byte vector. Return byte vector on success and
    /// a string on error.
    pub fn read_file(&mut self) -> Result<Vec<u8>, String> {
        let metadata = self.get_metadata()?;
        let mut result = vec![0u8; metadata.len() as usize];
        self.read_into_vec(&mut result)?;

        Ok(result)
    }

    /// Read `self`'s contents into a byte vector. Return byte vector on success and
    /// a string on error.
    pub fn get_metadata(&mut self) -> Result<Metadata, String> {
        match self.file.metadata() {
            Ok(r) => Ok(r),
            Err(e) => Err(format!("Could not read metadata for {}: {}", self.path, e)),
        }
    }

    /// Read `self`'s contents into a byte buffer, reading as many bytes as
    /// necessary to fill that buffer. Return void on success and
    /// a string on error.
    /// # Arguments
    /// * `buf` - Byte buffer to read `self` into.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<(), String> {
        match self.file.read_exact(buf) {
            Ok(r) => Ok(()),
            Err(e) => Err(format!("Could not read buffer from {}: {}", self.path, e)),
        }
    }

    /// Write `buf`'s contents into a self (as binary data).
    /// Return void on success and string on error.
    /// # Arguments
    /// * `buf` - Byte buffer to write to `self`.
    pub fn write_buf(&mut self, buf: &[u8]) -> Result<(), String> {
        match self.file.write_all(buf) {
            Ok(r) => Ok(()),
            Err(e) => Err(format!(
                "Could not write byte buffer to {}: {}",
                self.path, e
            )),
        }
    }

    /// Write `buf`'s contents into a self (as binary data).
    /// Return void on success and  string on error.
    /// # Arguments
    /// * `buf` - Byte vector to write to `self`.
    pub fn write_vec(&mut self, buf: &Vec<u8>) -> Result<(), String> {
        match self.file.write_all(&buf[..]) {
            Ok(r) => Ok(()),
            Err(e) => Err(format!(
                "Could not write byte buffer to {}: {}",
                self.path, e
            )),
        }
    }
}
