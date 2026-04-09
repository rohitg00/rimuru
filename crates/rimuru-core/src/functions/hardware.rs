use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};

use super::sysutil::{kv_err, parse_meminfo_kb, parse_vm_stat_value, run_cmd};
use crate::models::hardware::{assess_fit, local_equivalents};
use crate::models::{
    AccelBackend, CatalogEntry, CatalogModel, FitLevel, GpuInfo, HardwareInfo, LocalModelAdvisory,
    ModelInfo,
};
use crate::state::StateKV;

const CATALOG_JSON: &str = include_str!("../../data/local_models.json");

pub fn register(iii: &III, kv: &StateKV) {
    register_detect(iii, kv);
    register_get(iii, kv);
    register_assess(iii, kv);
    register_catalog(iii, kv);
}

fn register_detect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.hardware.detect".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let hw = detect_hardware().await;
                kv.set("hardware", "system_info", &hw)
                    .await
                    .map_err(kv_err)?;
                Ok(json!({"hardware": hw}))
            }
        },
    );
}

fn register_get(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.hardware.get".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let hw: Option<HardwareInfo> =
                    kv.get("hardware", "system_info").await.map_err(kv_err)?;
                match hw {
                    Some(h) => Ok(json!({"hardware": h})),
                    None => {
                        let detected = detect_hardware().await;
                        kv.set("hardware", "system_info", &detected)
                            .await
                            .map_err(kv_err)?;
                        Ok(json!({"hardware": detected}))
                    }
                }
            }
        },
    );
}

fn register_assess(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.advisor.assess".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let hw: Option<HardwareInfo> =
                    kv.get("hardware", "system_info").await.map_err(kv_err)?;

                let hw = match hw {
                    Some(h) => h,
                    None => {
                        let detected = detect_hardware().await;
                        kv.set("hardware", "system_info", &detected)
                            .await
                            .map_err(kv_err)?;
                        detected
                    }
                };

                let models: Vec<ModelInfo> = kv.list("model_info").await.map_err(kv_err)?;

                let cost_summary: Option<Value> =
                    kv.get("cost_agent", "summary").await.map_err(kv_err)?;

                let by_model = cost_summary
                    .as_ref()
                    .and_then(|v| v.get("by_model"))
                    .and_then(|v| v.as_array());

                let equivalents = local_equivalents();
                let mut advisories: Vec<LocalModelAdvisory> = Vec::new();

                for eq in &equivalents {
                    let model = models.iter().find(|m| m.id == eq.api_model_id);
                    let model_name = model
                        .map(|m| m.name.clone())
                        .unwrap_or_else(|| eq.api_model_id.to_string());

                    let api_cost_spent = by_model
                        .and_then(|arr| {
                            arr.iter().find(|m| {
                                m.get("model_id")
                                    .and_then(|id| id.as_str())
                                    .map(|id| id == eq.api_model_id)
                                    .unwrap_or(false)
                            })
                        })
                        .and_then(|m| m.get("total_cost").and_then(|c| c.as_f64()))
                        .unwrap_or(0.0);

                    let (fit_level, best_quant, est_vram, est_tok_s) = assess_fit(&hw, eq.params_b);

                    let can_run = fit_level != FitLevel::TooTight;
                    let potential_savings = if can_run { api_cost_spent } else { 0.0 };

                    advisories.push(LocalModelAdvisory {
                        model_id: eq.api_model_id.to_string(),
                        model_name,
                        can_run_locally: can_run,
                        fit_level,
                        best_quantization: best_quant,
                        estimated_vram_mb: est_vram,
                        estimated_tok_per_sec: est_tok_s,
                        local_equivalent: Some(eq.local_name.to_string()),
                        api_cost_spent,
                        potential_savings,
                    });
                }

                kv.set("advisor", "assessments", &advisories)
                    .await
                    .map_err(kv_err)?;

                Ok(json!({"advisories": advisories}))
            }
        },
    );
}

fn register_catalog(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.advisor.catalog".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let hw: Option<HardwareInfo> =
                    kv.get("hardware", "system_info").await.map_err(kv_err)?;

                let hw = match hw {
                    Some(h) => h,
                    None => {
                        let detected = detect_hardware().await;
                        kv.set("hardware", "system_info", &detected)
                            .await
                            .map_err(kv_err)?;
                        detected
                    }
                };

                let filter = input
                    .get("filter")
                    .and_then(|v| v.as_str())
                    .unwrap_or("runnable");

                let catalog: Vec<CatalogModel> =
                    serde_json::from_str(CATALOG_JSON).map_err(|e| {
                        iii_sdk::IIIError::Handler(format!("Failed to parse catalog: {}", e))
                    })?;

                let mut entries: Vec<CatalogEntry> = Vec::new();

                for model in &catalog {
                    let (fit_level, best_quant, est_vram, est_tok_s) =
                        assess_fit(&hw, model.params_b);
                    let can_run = fit_level != FitLevel::TooTight;

                    if filter == "runnable" && !can_run {
                        continue;
                    }

                    entries.push(CatalogEntry {
                        name: model.name.clone(),
                        provider: model.provider.clone(),
                        params_b: model.params_b,
                        context_length: model.context_length,
                        use_case: model.use_case.clone(),
                        architecture: model.architecture.clone(),
                        capabilities: model.capabilities.clone(),
                        hf_downloads: model.hf_downloads,
                        fit_level,
                        can_run,
                        best_quantization: best_quant,
                        estimated_vram_mb: est_vram,
                        estimated_tok_per_sec: est_tok_s,
                    });
                }

                let total = entries.len();
                let perfect = entries
                    .iter()
                    .filter(|e| e.fit_level == FitLevel::Perfect)
                    .count();
                let good = entries
                    .iter()
                    .filter(|e| e.fit_level == FitLevel::Good)
                    .count();
                let marginal = entries
                    .iter()
                    .filter(|e| e.fit_level == FitLevel::Marginal)
                    .count();

                Ok(json!({
                    "entries": entries,
                    "total": total,
                    "summary": {
                        "perfect": perfect,
                        "good": good,
                        "marginal": marginal,
                        "catalog_size": catalog.len()
                    }
                }))
            }
        },
    );
}

async fn detect_hardware() -> HardwareInfo {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    if cfg!(target_os = "macos") {
        detect_macos(&os, &arch).await
    } else if cfg!(target_os = "linux") {
        detect_linux(&os, &arch).await
    } else {
        HardwareInfo {
            cpu_cores: std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1),
            cpu_brand: "Unknown".into(),
            total_ram_mb: 0,
            available_ram_mb: 0,
            gpu: None,
            backend: AccelBackend::CpuX86,
            os,
            arch,
        }
    }
}

async fn detect_macos(os: &str, arch: &str) -> HardwareInfo {
    let total_bytes: u64 = run_cmd("sysctl", &["-n", "hw.memsize"])
        .await
        .trim()
        .parse()
        .unwrap_or(0);
    let total_ram_mb = total_bytes / (1024 * 1024);

    let cpu_cores: u32 = run_cmd("sysctl", &["-n", "hw.ncpu"])
        .await
        .trim()
        .parse()
        .unwrap_or(1);

    let cpu_brand = run_cmd("sysctl", &["-n", "machdep.cpu.brand_string"])
        .await
        .trim()
        .to_string();

    let is_apple_silicon = arch.contains("aarch64") || arch.contains("arm");

    let (gpu, backend) = if is_apple_silicon {
        let gpu_text = run_cmd("system_profiler", &["SPDisplaysDataType"]).await;
        let gpu_name = gpu_text
            .lines()
            .find(|l| l.contains("Chipset Model:"))
            .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
            .unwrap_or_else(|| "Apple Silicon".into());

        (
            Some(GpuInfo {
                name: gpu_name,
                vram_mb: total_ram_mb,
                count: 1,
            }),
            AccelBackend::Metal,
        )
    } else {
        (None, AccelBackend::CpuX86)
    };

    let available_ram_mb = get_macos_available_ram(total_ram_mb).await;

    HardwareInfo {
        cpu_cores,
        cpu_brand,
        total_ram_mb,
        available_ram_mb,
        gpu,
        backend,
        os: os.into(),
        arch: arch.into(),
    }
}

async fn get_macos_available_ram(total_mb: u64) -> u64 {
    let output = run_cmd("vm_stat", &[]).await;
    let page_size: u64 = 16384;
    let mut free: u64 = 0;
    let mut inactive: u64 = 0;
    let mut speculative: u64 = 0;

    for line in output.lines() {
        if line.contains("Pages free:") {
            free = parse_vm_stat_value(line);
        } else if line.contains("Pages inactive:") {
            inactive = parse_vm_stat_value(line);
        } else if line.contains("Pages speculative:") {
            speculative = parse_vm_stat_value(line);
        }
    }

    let available_bytes = (free + inactive + speculative) * page_size;
    let available_mb = available_bytes / (1024 * 1024);
    if available_mb > 0 {
        available_mb
    } else {
        total_mb / 2
    }
}

async fn detect_linux(os: &str, arch: &str) -> HardwareInfo {
    let meminfo = tokio::fs::read_to_string("/proc/meminfo")
        .await
        .unwrap_or_default();
    let mut total_kb: u64 = 0;
    let mut available_kb: u64 = 0;
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = parse_meminfo_kb(line);
        } else if line.starts_with("MemAvailable:") {
            available_kb = parse_meminfo_kb(line);
        }
    }

    let cpuinfo = tokio::fs::read_to_string("/proc/cpuinfo")
        .await
        .unwrap_or_default();
    let cpu_brand = cpuinfo
        .lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".into());

    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);

    let (gpu, backend) = detect_linux_gpu(arch).await;

    HardwareInfo {
        cpu_cores,
        cpu_brand,
        total_ram_mb: total_kb / 1024,
        available_ram_mb: available_kb / 1024,
        gpu,
        backend,
        os: os.into(),
        arch: arch.into(),
    }
}

async fn detect_linux_gpu(arch: &str) -> (Option<GpuInfo>, AccelBackend) {
    let nvidia = run_cmd(
        "nvidia-smi",
        &[
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ],
    )
    .await;

    if !nvidia.is_empty() && !nvidia.contains("not found") && !nvidia.contains("error") {
        let parts: Vec<&str> = nvidia.trim().split(',').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim().to_string();
            let vram: u64 = parts[1].trim().parse().unwrap_or(0);
            return (
                Some(GpuInfo {
                    name,
                    vram_mb: vram,
                    count: 1,
                }),
                AccelBackend::Cuda,
            );
        }
    }

    let rocm = run_cmd("rocm-smi", &["--showmeminfo", "vram"]).await;
    if !rocm.is_empty() && !rocm.contains("not found") {
        return (
            Some(GpuInfo {
                name: "AMD GPU".into(),
                vram_mb: 8192,
                count: 1,
            }),
            AccelBackend::Rocm,
        );
    }

    let cpu_backend = if arch.contains("arm") || arch.contains("aarch64") {
        AccelBackend::CpuArm
    } else {
        AccelBackend::CpuX86
    };
    (None, cpu_backend)
}
