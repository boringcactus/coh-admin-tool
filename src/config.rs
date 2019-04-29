use std::{env, fmt};
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use toml;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub sql_login: String,
    pub port: u16,
    pub register_secret: String,
    pub promote_secret: String,
    pub resetpw_secret: String,
}

#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    De(toml::de::Error),
    Ser(toml::ser::Error),
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::De(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigError::Ser(e)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(ref e) => f.write_fmt(format_args!("{}", e)),
            ConfigError::De(ref e) => f.write_fmt(format_args!("{}", e)),
            ConfigError::Ser(ref e) => f.write_fmt(format_args!("{}", e)),
        }
    }
}

impl Error for ConfigError {}

type Result<T> = std::result::Result<T, ConfigError>;

fn prompt<T: FromStr>(text: &str) -> T where <T as std::str::FromStr>::Err: std::fmt::Display {
    loop {
        let mut buf = String::new();
        print!("{}: ", text);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().to_owned();
        match buf.parse() {
            Ok(x) => return x,
            Err(e) => println!("Invalid {}: {}", text, e),
        }
    }
}

fn load_from_file() -> Result<Config> {
    let mut path = env::current_exe()?;
    path.set_extension("toml");
    let toml_data = fs::read(path)?;
    let result = toml::from_slice(&toml_data)?;
    Ok(result)
}

fn save_to_file(conf: &Config) -> Result<()> {
    let mut path = env::current_exe()?;
    path.set_extension("toml");
    fs::write(&path, toml::to_vec(conf)?)?;
    println!("Configuration saved to {}", path.display());
    Ok(())
}

pub fn load() -> Config {
    if let Ok(conf) = load_from_file() {
        return conf;
    }
    println!("Steal SQL login from account_server.cfg. Start with DRIVER= and end with ;, leave out the quotes.");
    let sql_login = prompt("SQL login");
    let port = prompt("Port");
    let register_secret = prompt("Registration secret");
    let promote_secret = prompt("Promotion secret");
    let resetpw_secret = prompt("Password reset secret");
    let conf = Config {
        sql_login,
        port,
        register_secret,
        promote_secret,
        resetpw_secret,
    };
    let _ = save_to_file(&conf);
    conf
}
