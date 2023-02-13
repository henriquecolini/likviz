use std::{
    collections::HashMap,
    io::Read,
    process::{Command, Stdio},
    str,
};


use fancy_regex::Regex;
use lazy_static::lazy_static;
use tempfile::NamedTempFile;

use crate::log::{label_err, label_inf, label_wrn, log_wrn, LogExpect};

fn likwid_run(program: &str, args: &[&str]) -> String {
    let tmp_out = NamedTempFile::new().label_expect(program, "Falha ao criar arquivo temporário");
    let mut tmp_out_r = tmp_out
        .reopen()
        .label_expect(program, "Falha ao ler arquivo temporário");
    let tmp_out = Stdio::from(tmp_out.into_file());

    let tmp_err = NamedTempFile::new().label_expect(program, "Falha ao criar arquivo temporário");
    let mut tmp_err_r = tmp_err
        .reopen()
        .label_expect(program, "Falha ao ler arquivo temporário");
    let tmp_err = Stdio::from(tmp_err.into_file());

    let mut child = Command::new(program)
        .args(args)
        .stdout(tmp_out)
        .stderr(tmp_err)
        .spawn()
        .label_expect(program, "Falha na execução");

    label_inf!(program, "Executando: PID {}", child.id());

    let status = child
        .wait()
        .label_expect(program, "Falha ao aguardar execução");

    let mut stdout = vec![];
    let mut stderr = vec![];

    tmp_out_r
        .read_to_end(&mut stdout)
        .label_expect(program, "Falha ao ler saída");
    tmp_err_r
        .read_to_end(&mut stderr)
        .label_expect(program, "Falha ao ler erros");

    match str::from_utf8(&stderr) {
        Ok(stderr) => {
            for line in stderr.lines() {
                label_err!(program, "{}", line);
            }
        }
        Err(_) => {}
    }

    if status.success() {
        label_inf!(program, "Finalizado: {}", status);
    } else {
        label_wrn!(program, "Finalizado: {}", status);
    }

    String::from_utf8(stdout).label_expect(program, "Saída não UTF-8")
}

pub struct Metric {
    pub key: String,
    pub value: f64,
}

pub struct RegionMetrics {
    pub title: String,
    pub metrics: Vec<Metric>,
}

lazy_static! {
    static ref MATCHER: Regex = Regex::new(
        r"(?ms)TABLE,Region ([^\n]*?),[^\n]*?\nMetric,.*?\n(.*?)(?=^STRUCT|^TABLE|^Region|\z)"
    )
    .log_expect("Falha ao construir regex");
}

fn likwid_extract(input: String) -> HashMap<String, RegionMetrics> {
    let mut regions = HashMap::new();

    for cap in MATCHER.captures_iter(&input) {
        let cap = cap.log_expect("??");
        let title = cap[1].to_string();
        let mut metrics = Vec::new();

        for line in cap[2].lines() {
            let tokens: Vec<&str> = line.split(',').collect();
            if tokens.len() < 2 {
                log_wrn!("Formato inválido de dados: '{}'", line);
                continue;
            }
            let metric = tokens[0].to_string();
            if let Ok(value) = tokens[1].parse::<f64>() {
                metrics.push(Metric { key: metric, value });
            } else {
                log_wrn!("Valor não numérico: {}", tokens[1]);
            }
        }
        regions.insert(title.clone(), RegionMetrics { title, metrics });
    }
    regions
}

pub fn perfctr(group: &str, core: u8, program: &[&str]) -> HashMap<String, RegionMetrics> {
    let core_str: &str = &core.to_string();
    let args: &[&str] = &["-O", "-C", core_str, "-g", group, "-m"];
    let args: &[&str] = &[args, program].concat();
    let output = likwid_run("likwid-perfctr", args);
    let output = likwid_extract(output);
    label_inf!(
        "likwid-perfctr",
        "Regiões extraídas: {}",
        output
            .iter()
            .map(|(id, region)| format!("{} ({})", id, region.metrics.len()))
            .collect::<Vec<String>>()
            .join(", ")
    );
    output
}
