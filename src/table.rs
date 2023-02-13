use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    str,
};

use colored::{ColoredString, Colorize};
use tempfile::NamedTempFile;

use crate::{
    config::Config,
    likwid::RegionMetrics,
    log::{label_err, label_inf, label_wrn, log_inf, log_wrn, LogExpect},
};

pub struct MetricSet(pub HashMap<String, Vec<f64>>);
pub struct RegionSet(pub HashMap<String, MetricSet>);

pub fn update_regset(regset: &mut RegionSet, regions: &HashMap<String, RegionMetrics>) {
    for (name, metrics) in regions {
        if !regset.0.contains_key(name) {
            regset.0.insert(name.to_owned(), MetricSet(HashMap::new()));
        }
        let reg = regset.0.get_mut(name).unwrap();
        for metric in &metrics.metrics {
            if !reg.0.contains_key(&metric.key) {
                reg.0.insert(metric.key.to_owned(), vec![]);
            }
            let val = reg.0.get_mut(&metric.key).unwrap();
            val.push(metric.value);
        }
    }
}

pub struct TableColumn {
    title: String,
    values: Vec<f64>,
}

pub struct Table {
    label: String,
    title: String,
    columns: Vec<TableColumn>,
}

pub fn create_table(
    label: &str,
    title: &str,
    regset: &RegionSet,
    regions: &Vec<String>,
    metrics: &Vec<String>,
    n_values: &[u32],
) -> Table {
    let mut table = Table {
        label: label.to_owned(),
        title: title.to_owned(),
        columns: vec![],
    };
    let mut missing_metrics = HashSet::new();
    table.columns.push(TableColumn {
        title: "n".to_owned(),
        values: n_values.iter().map(|x| *x as f64).collect(),
    });
    for region in regions {
        let reg = regset.0.get(region);
        if let Some(reg) = reg {
            for metric in metrics {
                if missing_metrics.contains(metric) {
                    continue;
                }
                if let Some(values) = reg.0.get(metric) {
                    table.columns.push(TableColumn {
                        title: format!("{region} ({metric})"),
                        values: values.to_owned(),
                    });
                } else {
                    log_wrn!("Métrica '{}' não encontrada", metric);
                    missing_metrics.insert(metric.to_owned());
                }
            }
        } else {
            log_wrn!("Região '{}' não encontrada", region);
        }
    }
    log_inf!("Tabela criada com {} colunas", table.columns.len());
    table
}

fn write_csv(table: &Table, file: &mut File) -> std::io::Result<()> {
    let titles = table
        .columns
        .iter()
        .map(|x| x.title.replace('_', " "))
        .collect::<Vec<String>>()
        .join(",");
    write!(file, "{}\n", titles)?;
    for i in 0..table.columns[0].values.len() {
        let values = table
            .columns
            .iter()
            .map(|x| match x.values.get(i) {
                Some(val) => val.to_string(),
                None => "".to_owned(),
            })
            .collect::<Vec<String>>()
            .join(",");
        write!(file, "{}\n", values)?;
    }
    Ok(())
}

fn prepare_export_path(table: &Table, config: &Config, file_type: &str, name: &str) -> PathBuf {
    let dir = PathBuf::from(&config.output_path)
        .join(file_type)
        .join(&table.label);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).log_expect(&format!("Falha ao criar diretório: {:?}", dir));
    }
    dir.join(name.to_owned() + "." + file_type)
}

fn colored_filename(path: &PathBuf) -> ColoredString {
    if let Some(filename) = path.file_name() {
        if let Some(filename) = filename.to_str() {
            return filename.color("green");
        }
    }
    return "??".color("red");
}

pub fn export_csv(table: &Table, config: &Config) {
    let csv_path = prepare_export_path(table, config, "csv", &table.title);
    log_inf!("Exportando planilha CSV: {}", colored_filename(&csv_path));
    let mut csv_file = File::create(csv_path).log_expect("Falha ao criar planilha CSV");
    write_csv(table, &mut csv_file).log_expect("Falha ao escrever planilha CSV");
}

pub fn export_png(table: &Table, config: &Config) {
    let png_path = prepare_export_path(table, config, "png", &table.title);
    log_inf!("Exportando imagem PNG: {}", colored_filename(&png_path));
    let mut tmp_csv = NamedTempFile::new().log_expect("Falha ao criar arquivo temporário");
    write_csv(table, tmp_csv.as_file_mut()).log_expect("Falha ao escrever arquivo");
    let script = match table.columns.len() {
        0 => {
            log_wrn!("Não há colunas para gerar imagem PNG");
            return;
        }
        1 => {
            format!(
                r"set datafile separator ',';
                set term png;
                set output '{}';
                FILE = '{}';
                plot for [col=1:1] FILE u 1:col w l title columnheader(col);
                quit;",
                png_path.to_str().unwrap(),
                tmp_csv.path().to_str().unwrap()
            )
        }
        n => {
            format!(
                r"set datafile separator ',';
                set term png;
                set output '{}';
                FILE = '{}';
                plot for [col={}:{}] FILE u {}:col w l title columnheader(col);
                quit;",
                png_path.to_str().unwrap(),
                tmp_csv.path().to_str().unwrap(),
                2,
                n,
                1
            )
        }
    };
    let child = Command::new("gnuplot")
        .args(["-e", &script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .label_expect("gnuplot", "Falha na execução");

    label_inf!("gnuplot", "Executando: PID {}", child.id());

    let output = child
        .wait_with_output()
        .label_expect("gnuplot", "Falha ao aguardar execução");

    match str::from_utf8(&output.stderr) {
        Ok(stderr) => {
            for line in stderr.lines() {
                label_err!("gnuplot", "{}", line);
            }
        }
        Err(_) => {}
    }

    if output.status.success() {
        label_inf!("gnuplot", "Finalizado: {}", output.status);
    } else {
        label_wrn!("gnuplot", "Finalizado: {}", output.status);
    }
}
