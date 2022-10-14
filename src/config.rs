// RISC II emulator configuration.
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

// Struct definitions.

extern crate serde;
extern crate serde_derive;
extern crate toml;

use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use util::{concat_paths, get_home_nofail, Result};

use berr;

use self::serde_derive::Deserialize;

/// Configuration of the emulator.
#[derive(Deserialize)]
pub struct Config {
    /// Amount of memory the system will have.
    #[serde(default = "default_mem")]
    mem: u32,
    /// Number of CPUs the system will have.
    #[serde(default = "default_ncpu")]
    ncpu: u32,
    /// Path to the configuration file.
    #[serde(skip_deserializing)]
    config_file_path: String,
    /// Path to the system cache directory.
    #[serde(default = "default_cache")]
    cache_path: String,
    /// The clock rate (in hertz).
    #[serde(default = "default_clock_rate")]
    clock_rate: u64,
    /// Width of the window.
    #[serde(default = "default_width")]
    win_width: u32,
    /// Height of the window.
    #[serde(default = "default_height")]
    win_height: u32,
    /// Width of the window.
    #[serde(default = "default_width")]
    debug_win_width: u32,
    /// Height of the window.
    #[serde(default = "default_height")]
    debug_win_height: u32,
    /// True if in debug mode, false otherwise.
    #[serde(default = "default_debug_mode")]
    debug_mode: bool,
}

// Struct impls.

impl Config {
    /// Create a new configuration object (with default settings) on success and a string on error.
    pub fn new() -> Result<Config> {
        let home_dir = get_home_nofail();
        // Find a configuration path specified on the command line.
        let config_path = match env::var("XDG_CONFIG_HOME") {
            Ok(v) => format!("{}", v),
            Err(e) => format!("{}", home_dir),
        };

        Ok(Config {
            mem: 0,
            ncpu: 0,
            config_file_path: concat_paths(
                &config_path,
                &".config/riscii/config.toml".to_string(),
            )?,
            clock_rate: 0,
            cache_path: String::new(),
            win_width: 0,
            win_height: 0,
            debug_win_width: 0,
            debug_win_height: 0,
            debug_mode: false,
        })
    }

    /// Create an initialized configuration object on success and a string on error.
    pub fn init() -> Result<Config> {
        let mut config = Self::new()?;
        let args: Vec<String> = env::args().collect();
        // Look for custom config file location first. Read it, then override with cmd args.
        let cmd_config_file = config.find_cmd_config_path(&args)?;

        config.config_file_path = match cmd_config_file {
            None => config.config_file_path,
            Some(s) => s.to_string(),
        };

        config.read_config_file()?;
        config.parse_cmd_args(&args)?;
        Ok(config)
    }

    /// Read the user's configuration file and update configuration state
    /// (default ~/.config/riscii/config.toml). Return void on success and a
    /// string on error.
    fn read_config_file(&mut self) -> Result<()> {
        // TODO do not exit if config.toml does not exist
        // TODO get ~ in paths to expand
        // Keep the data we want to survive the assignment.
        let config_file_path = self.config_file_path.clone();
        *self = match toml::from_str(&match fs::read_to_string(Path::new(&config_file_path)) {
            Err(e) => return berr!(format!("Could not read {}, {}", config_file_path, e)),
            Ok(r) => r,
        }) {
            Err(e) => {
                return berr!(format!(
                    "Could not parse config file {}, {}",
                    config_file_path, e
                ))
            }
            Ok(r) => r,
        };

        self.config_file_path = config_file_path;

        Ok(())
    }

    /// Parse CMD arguments for configuration file path. Return path on
    /// success and string on error.
    /// # Arguments
    /// * `args` - CMD argument vector.
    fn find_cmd_config_path(&self, args: &Vec<String>) -> Result<Option<String>> {
        for (i, arg) in args.iter().enumerate() {
            match arg.as_str() {
                "--config_path" => {
                    return Ok(Some(
                        args_get_next_arg(&args, i, &format!("config_path"))?.clone(),
                    ))
                }
                _ => (),
            }
        }
        Ok(None)
    }

    /// Parse CMD args and update configuration state. Return void on success
    /// and a string on error.
    /// # Arguments
    /// * `args` - CMD argument vector.
    fn parse_cmd_args(&mut self, args: &Vec<String>) -> Result<()> {
        let mut skips = 1i32;
        for (i, arg) in args.iter().enumerate() {
            if skips > 0 {
                skips -= 1;
                continue;
            }

            match arg.as_str() {
                "--mem" => {
                    self.mem = args_get_next_uint(&args, i, &format!("mem"))?;
                    skips += 1;
                }
                "--ncpu" => {
                    self.ncpu = args_get_next_uint(&args, i, &format!("ncpu"))?;
                    skips += 1;
                }
                "--cache_path" => {
                    self.cache_path = args_get_next_arg(&args, i, &format!("cache_path"))?.clone();
                    skips += 1;
                }
                // Skip this argument since it is special.
                "--config_path" => {
                    args_get_next_arg(&args, i, &format!("config_path"))?;
                    skips += 1;
                }
                "--win_width" => {
                    self.win_width = args_get_next_uint(&args, i, &format!("win_width"))?;
                    skips += 1;
                }
                "--win_height" => {
                    self.win_height = args_get_next_uint(&args, i, &format!("win_height"))?;
                    skips += 1;
                }
                _ => {
                    println!(
                        "Usage: riscii [OPTIONS]
--config_path       Path to configuration file (default=~/.config/riscii/)
--config_file_path  Path to the configuration file (default=~/.config/riscii/config.toml)
--mem               Size of memory (in megabytes) (default=512)
--ncpu              Number of cores to emulate (default=1)
"
                    );
                    return berr!(format!("Invalid command line argument: {}", arg));
                }
            }
        }
        Ok(())
    }

    // Getters.

    /// Get the user's configured window width.
    pub fn get_debug_win_width(&self) -> u32 {
        self.debug_win_width
    }

    /// Get the user's configured window height.
    pub fn get_debug_win_height(&self) -> u32 {
        self.debug_win_height
    }

    /// Get the user's configured window width.
    pub fn get_win_width(&self) -> u32 {
        self.win_width
    }

    /// Get the user's configured window height.
    pub fn get_win_height(&self) -> u32 {
        self.win_height
    }

    /// Get the user's configured memory size.
    pub fn get_mem_size(&self) -> u32 {
        self.mem
    }

    /// Get the user's configured number of CPUs.
    pub fn get_ncpus(&self) -> u32 {
        self.ncpu
    }

    /// Get the debug mode option.
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }
}

// Local functions.

/// Check the argument vector to make sure it has at least one more string
/// after the current argument. Return void on success and a string on error.
/// # Arguments
/// * `args` - CMD argument vector.
/// * `i` - Index of the current argument.
/// * `what` - String describing the current argument (for error message).
fn args_check_size(args: &Vec<String>, i: usize, what: &String) -> Result<()> {
    if i >= args.len() {
        berr!(format!(
            "Invalid command line argument: {} takes an argument.",
            what
        ))
    } else {
        Ok(())
    }
}

/// Get the next argument in the argument vector as a string. Return next
/// string on success and a string on error.
/// # Arguments
/// * `args` - CMD argument vector.
/// * `i` - Index of the current argument.
/// * `what` - String describing the current argument (for error message).
fn args_get_next_arg<'a>(args: &'a Vec<String>, i: usize, what: &String) -> Result<&'a String> {
    args_check_size(&args, i, &what)?;
    Ok(&args[i + 1])
}

/// Get the next argument in the argument vector as a u32. Return
/// u32 on success and a string on error.
/// # Arguments
/// * `args` - CMD argument vector.
/// * `i` - Index of the current argument.
/// * `what` - String describing the current argument (for error message).
fn args_get_next_uint(args: &Vec<String>, i: usize, what: &String) -> Result<u32> {
    args_check_size(&args, i, &what)?;
    Ok(match args[i + 1].parse::<u32>() {
        core::result::Result::Ok(u) => u,
        core::result::Result::Err(e) => {
            return berr!(format!(
                "Invalid command line argument for {}: {}, err: {}.",
                what,
                args[i + 1],
                e
            ))
        }
    })
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Number of cpus: {}
Memory (MB): {}
Configuration file: {}
Cache Directory: {}
Window dimensions: ({}, {})",
            self.ncpu,
            self.mem,
            self.config_file_path,
            self.cache_path,
            self.win_width,
            self.win_height
        )
    }
}

// Local functions.

// Default functions for serde.

fn default_mem() -> u32 {
    64
}

fn default_ncpu() -> u32 {
    1
}

fn default_cache() -> String {
    let home_dir = get_home_nofail();

    let cache_dir = ".cache/riscii".to_string();
    match env::var("XDG_CACHE_HOME") {
        Ok(v) => concat_paths(&v, &cache_dir).unwrap(),
        Err(v) => concat_paths(&home_dir, &cache_dir).unwrap(),
    }
}

fn default_width() -> u32 {
    1200
}

fn default_height() -> u32 {
    900
}

fn default_debug_mode() -> bool {
    true
}

fn default_clock_rate() -> u64 {
    5_000_000
}
