use std::env;
use std::path;
use std::fs;
use std::io::Write;
use std::fmt;

use config;


const CNTR_HOST: &str = "0.0.0.0";
const CTNR_TARGET_DIR: &str = "/usr/share/www";
const FILE_SERVER_JSON_TARGET: &str = "file-server.json";
const PODMAN_COMPOSE_TARGET: &str = "file-server.podman-compose.yml";

const FAILED_TO_CONVERT_CONFIG: &str = "failed to convert config into string";
const FAILED_TO_CREATE_CONFIG: &str = "failed to create config";
const FAILED_TO_WRITE_CONFIG: &str = "failed to write config to disk";

const PODMAN_COMPOSE_NOT_FOUND: &str = "podman-compose template was not found";
const FAILED_TO_CONVERT_PODMAN_COMPOSE: &str = "failed to convert podman-compose template to string";
const FAILED_TO_CREATE_PODMAN_COMPOSE: &str = "failed to create podman-compose";
const FAILED_TO_WRITE_PODMAN_COMPOSE: &str = "failed to wright podman-compose to disk";

const FAILED_TO_PARSE_CTNR_PATH: &str = "failed to create container path";


pub struct ContainerError {
    message: String,
}

impl ContainerError {
    pub fn new(msg: &str) -> ContainerError {
        ContainerError { message: msg.to_string() }
    }
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}


pub fn get_pathbuff_from_args(index: usize) -> Option<path::PathBuf> {
    match env::args().nth(index) {
        Some(c) => Some(path::PathBuf::from(c)),
        _ => None,
    }
}

pub fn create_container_config(
    config: &config::Config,
) -> Result<config::Config, ContainerError> {
    let dest = path::PathBuf::from(CTNR_TARGET_DIR);

    let fp_403 = match get_container_pathbuf(
        &config.directory,
        &config.filepath_403,
        &dest,
    ) {
        Ok(fp) => fp,
        Err(e) => return Err(e),
    };

    let fp_404 = match get_container_pathbuf(
        &config.directory,
        &config.filepath_404,
        &dest,
    ) {
        Ok(fp) => fp,
        Err(e) => return Err(e),
    };

    let fp_500 = match get_container_pathbuf(
        &config.directory,
        &config.filepath_500,
        &dest,
    ) {
        Ok(fp) => fp,
        Err(e) => return Err(e),
    };

    Ok(config::Config {
        host: CNTR_HOST.to_string(),
    	port: 3000,
    	directory: dest,
    	filepath_403: fp_403,
    	filepath_404: fp_404,
    	filepath_500: fp_500,
    })
}

fn get_container_pathbuf(
    directory: &path::PathBuf,
    filepath: &path::PathBuf,
    taraget_dir: &path::PathBuf,
) -> Result<path::PathBuf, ContainerError> {
    match filepath.strip_prefix(directory) {
        Ok(pb) => Ok(taraget_dir.join(pb)),
        _ => Err(ContainerError::new(FAILED_TO_PARSE_CTNR_PATH)),
    }
}

pub fn write_config(
    destination: &path::PathBuf,
    config: &config::Config,
) -> Result<(), ContainerError> {
    let mut target = path::PathBuf::from(destination);
    target.push(FILE_SERVER_JSON_TARGET);

    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_CONFIG)),
    };

    let config = match config::config_to_string(&config) {
        Ok(s) => s,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_CONFIG)),
    };

    match output.write_all(config.as_bytes()) {
        Ok(o) => Ok(o),
        _ => return Err(ContainerError::new(FAILED_TO_WRITE_CONFIG)),
    }
}

pub fn write_podman_compose(
    destination: &path::PathBuf,
    config: &config::Config,
    podman_compose_filepath: &path::PathBuf,
) -> Result<(), ContainerError> {
    let contents = match fs::read_to_string(podman_compose_filepath) {
        Ok(c) => c,
        _ => return Err(ContainerError::new(PODMAN_COMPOSE_NOT_FOUND)),
    };

    let directory =  match config.directory.to_str() {
        Some(n) => n,
        _ => return Err(ContainerError::new(FAILED_TO_CONVERT_PODMAN_COMPOSE)),
    };

    let mut target = path::PathBuf::from(destination);
    target.push(PODMAN_COMPOSE_TARGET);
    
    let mut output = match fs::File::create(target) {
        Ok(o) => o,
        _ => return Err(ContainerError::new(FAILED_TO_CREATE_PODMAN_COMPOSE)),
    };

    let podman_compose = contents
        .replace("{host}", &config.host)
        .replace("{port}", &config.port.to_string())
        .replace("{directory}", directory);

    match output.write_all(podman_compose.as_bytes()) {
        Ok(o) => Ok(o),
        _ => Err(ContainerError::new(FAILED_TO_WRITE_PODMAN_COMPOSE)),
    }
}
