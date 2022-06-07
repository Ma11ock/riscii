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
use std::fmt;

/// Configuration of the emulator.
pub struct Config {
    /// Amount of memory the system will have.
    mem: u32,
    /// Number of CPUs the system will have.
    ncpu: u32,
    config_path: String,
}

// Struct impls.

impl Config {
    pub fn new() -> Config {
        Config {
            mem: 0,
            ncpu: 0,
            config_path: format!(""),
        }
    }

    pub fn init() -> Result<Config, String> {
        let mut config = Self::new();
        let args: Vec<String> = env::args().collect();

        let mut skips = 1i32;
        for (i, arg) in args.iter().enumerate() {
            if skips > 0 {
                skips -= 1;
                continue;
            }

            match arg.as_str() {
                "--mem" => {
                    config.mem = args_get_next_uint(&args, i, &format!("mem"))?;
                    skips += 1;
                }
                "--ncpu" => {
                    config.ncpu = args_get_next_uint(&args, i, &format!("ncpu"))?;
                    skips += 1;
                }
                "--config" => {
                    config.config_path = args_get_next_arg(&args, i, &format!("config"))?.clone();
                    skips += 1;
                }
                _ => {
                    println!(
                        "Usage: riscii [OPTIONS]
--config    Path to configuration file (default=~/.config/riscii/config.toml)
--mem       Size of memory (in megabytes) (default=512)
--ncpu      Number of cores to emulate (default=1)
"
                    );
                    return Err(format!("Invalid command line argument: {}", arg));
                }
            }
        }

        Ok(config)
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
Configuration file: {}",
            self.ncpu, self.mem, self.config_path
        )
    }
}
