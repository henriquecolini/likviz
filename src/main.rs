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
    for target in &config.target_paths {
        log_inf!("Testando executável {}", target.green());
    }
    cpu::set_cpu_freq(cpu::CpuFreq::Performance);
    let start = Instant::now();
    for group in &config.groups {
        log_inf!("{}", format!("Testando grupo '{group}'").bold().green());
        for test in &config.tests {
            log_inf!("Teste: '{}'", test.label);
            for target in &config.target_paths {
                let mut program = Vec::with_capacity(1 + test.params.len());
                program.push(target.to_owned());
                program.extend(test.params.to_owned());
                let program: Vec<&str> = program.iter().map(|s| s.as_ref()).collect();
                let regions = likwid::perfctr(
                    group,
                    config.core,
                    &program,
                );
                update_regset(&mut regset, &regions);
            }
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
            let x_axis: Vec<&str> = config.tests.iter().map(|t| &t.label as &str).collect();
            let table = table::create_table(
                &regions.label,
                &tconfig.title,
                &regset,
                &regions.regions,
                &tconfig.metrics,
                &x_axis,
            );
            table::export_csv(&table, &config);
            table::export_png(&table, &config);
        }
    }
}
