use std::fs;

use crate::log::{LogExpect, log_inf};

const CPU_FREQ_PATH: &str = "/sys/devices/system/cpu/cpufreq/policy3/scaling_governor";

#[derive(Debug)]
pub enum CpuFreq {
    Powersave,
    Performance,
}

pub fn set_cpu_freq(freq: CpuFreq) {
    log_inf!("Definindo frequência do processador para: {:?}", freq);
    fs::write(
        CPU_FREQ_PATH,
        match freq {
            CpuFreq::Powersave => "powersave".to_owned(),
            CpuFreq::Performance => "performance".to_owned(),
        },
    )
    .log_expect("Falha ao definir a frequência do processador")
}
