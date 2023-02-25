use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::log::LogExpect;
use crate::log::log_inf;

#[derive(Debug, Deserialize)]
pub struct RegionConfig {
    pub label: String,
    pub regions: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct TableConfig {
    pub title: String,
    pub metrics: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct TestConfig {
    pub label: String,
    pub params: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub target_paths: Vec<String>,
    pub output_path: String,
    pub core: u8,
    pub tests: Vec<TestConfig>,
    pub groups: Vec<String>,
    pub regions: Vec<RegionConfig>,
    pub tables: Vec<TableConfig>,
}

fn relativize_targets(config: &Config, config_dir: &PathBuf) -> Vec<String> {
    let mut targets = Vec::new();
    for target in &config.target_paths {
        let relative = PathBuf::try_from(target).log_expect(&format!(
            "Caminho inválido para o executável de teste: {}",
            target
        ));
        targets.push(config_dir
            .join(relative)
            .canonicalize()
            .log_expect("Não foi possível encontrar o executável de teste. Rodou make?")
            .to_str()
            .unwrap()
            .into());
    }
    targets
}

fn relativize_output(config: &Config, config_dir: &PathBuf) -> String {
    let relative = PathBuf::try_from(&config.output_path).log_expect(&format!(
        "Caminho inválido para o diretório de saída: {}",
        config.output_path
    ));
    let absolute = config_dir.join(relative);
    fs::create_dir_all(&absolute).log_expect("Não foi possível criar o diretório de saída");
    absolute
        .canonicalize()
        .log_expect("Não foi possível encontrar o diretório de saída")
        .to_str()
        .unwrap()
        .into()
}

fn load_config_from(path: PathBuf) -> Config {
    let str = fs::read_to_string(path).log_expect("Falha ao ler arquivo de configurações");
    serde_json::from_str(&str).log_expect("Falha ao processar arquivo de configurações")
}

pub fn load_config(config_path: PathBuf) -> Config {
    log_inf!("Lendo configurações em {}", config_path.to_str().unwrap().green());
    let config_dir = config_path.parent().unwrap().to_path_buf();
    let mut config = load_config_from(config_path);
    config.target_paths = relativize_targets(&config, &config_dir);
    config.output_path = relativize_output(&config, &config_dir);
    config
}
