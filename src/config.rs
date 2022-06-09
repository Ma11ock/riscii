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

use std::env;
use std::env::JoinPathsError;
use std::ffi::OsString;
use std::fmt;
use std::path::Path;

/// Configuration of the emulator.
pub struct Config {
    /// Amount of memory the system will have.
    mem: u32,
    /// Number of CPUs the system will have.
    ncpu: u32,
    /// Path to the configuration directory.
    config_path: String,
    /// Path to the configuration file.
    config_file_path: String,
    /// Path to the system cache directory.
    cache_path: String,
}

// Struct impls.

impl Config {
    pub fn new() -> Result<Config, String> {
        let home_dir = match env::var("HOME") {
            Ok(v) => format!("{}", v),
            Err(e) => {
                eprintln!("$HOME is not set. Defaulting to current directory.");
                format!(
                    "{}",
                    match env::current_dir() {
                        Ok(r) => os_string_result_to_strings(r.into_os_string().into_string())?,
                        Err(e) => format!("{}", e),
                    }
                )
            }
        };

        let config_path = match env::var("XDG_CONFIG_HOME") {
            Ok(v) => format!("{}", v),
            Err(e) => format!("{}", home_dir),
        };

        let cache_dir = ".cache/riscii".to_string();
        Ok(Config {
            mem: 0,
            ncpu: 0,
            config_file_path: concat_paths(
                &config_path,
                &".config/riscii/config.toml".to_string(),
            )?,
            cache_path: match env::var("XDG_CACHE_HOME") {
                Ok(v) => concat_paths(&v, &cache_dir)?,
                Err(v) => concat_paths(&home_dir, &cache_dir)?,
            },
            config_path: config_path,
        })
    }

    pub fn init() -> Result<Config, String> {
        let mut config = Self::new()?;
        config.parse_cmd_args()?;
        config.read_config_file()?;
        Ok(config)
    }

    fn read_config_file(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn parse_cmd_args(&mut self) -> Result<(), String> {
        let args: Vec<String> = env::args().collect();

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
                "--config" => {
                    self.config_path = args_get_next_arg(&args, i, &format!("config"))?.clone();
                    skips += 1;
                }
                _ => {
                    println!(
                        "Usage: riscii [OPTIONS]
--config    Path to configuration file (default=~/.config/riscii/self.toml)
--mem       Size of memory (in megabytes) (default=512)
--ncpu      Number of cores to emulate (default=1)
"
                    );
                    return Err(format!("Invalid command line argument: {}", arg));
                }
            }
        }
        Ok(())
    }
}

// Local functions.

fn args_check_size(args: &Vec<String>, i: usize, what: &String) -> Result<(), String> {
    if i >= args.len() {
        Err(format!(
            "Invalid command line argument: {} takes an argument.",
            what
        ))
    } else {
        Ok(())
    }
}

fn args_get_next_arg<'a>(
    args: &'a Vec<String>,
    i: usize,
    what: &String,
) -> Result<&'a String, String> {
    args_check_size(&args, i, &what)?;
    Ok(&args[i + 1])
}

fn args_get_next_uint(args: &Vec<String>, i: usize, what: &String) -> Result<u32, String> {
    args_check_size(&args, i, &what)?;
    Ok(match args[i + 1].parse::<u32>() {
        core::result::Result::Ok(u) => u,
        core::result::Result::Err(e) => {
            return Err(format!(
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
Configuration directory: {}
Configuration file: {}
Cache Directory: {}",
            self.ncpu, self.mem, self.config_path, self.config_file_path, self.cache_path
        )
    }
}

// Local functions.

fn os_string_result_to_strings(r: Result<String, OsString>) -> Result<String, String> {
    match r {
        Err(e) => Err(match e.into_string() {
            Ok(s) => s,
            Err(ee) => "Could not coerce OS string into utf8 string".to_string(),
        }),
        Ok(rr) => Ok(rr.to_string()),
    }
}

fn concat_paths(base: &String, rest: &String) -> Result<String, String> {
    let p = Path::new(&base).join(&rest);
    match p.to_str() {
        None => Err(format!("{} and {} joined is not valid utf8", base, rest)),
        Some(s) => Ok(s.to_string()),
    }
}
