use clap::Parser;
use colored::Colorize;

use std::{collections::HashMap, time::Instant, path::PathBuf};

use crate::{
    log::{log_inf},
    table::{update_regset, RegionSet},
};

mod config;
mod cpu;
mod likwid;
mod log;
mod table;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_path: PathBuf
}

fn main() {
    let args = Args::parse();
    let config = config::load_config(args.config_path);
    let mut regset = RegionSet(HashMap::new());
    log_inf!("Testando executável {}", config.target_path.green());
    cpu::set_cpu_freq(cpu::CpuFreq::Performance);
    let start = Instant::now();
    for group in &config.groups {
        log_inf!("{}", format!("Testando grupo '{group}'").bold().green());
        for n in &config.sizes.values {
            log_inf!("n = {}", n);
            let regions = likwid::perfctr(
                group,
                config.core,
                &[&config.target_path, &config.sizes.flag, &n.to_string()],
            );
            update_regset(&mut regset, &regions);
        }
    }
    let duration = start.elapsed();
    log_inf!(
        "{}",
        format!("Testes concluídos em {duration:?}")
            .bold()
            .green()
    );
    cpu::set_cpu_freq(cpu::CpuFreq::Powersave);
    log_inf!("Gerando tabelas");
    for tconfig in &config.tables {
        log_inf!("{}", format!("Exportando tabelas de '{}'", &tconfig.title).bold().green());
        for regions in &config.regions {
            log_inf!("Regiões: {:?} ({})", &regions.regions, &regions.label);
            let table = table::create_table(
                &regions.label,
                &tconfig.title,
                &regset,
                &regions.regions,
                &tconfig.metrics,
                &config.sizes.values,
            );
            table::export_csv(&table, &config);
            table::export_png(&table, &config);
        }
    }
}
