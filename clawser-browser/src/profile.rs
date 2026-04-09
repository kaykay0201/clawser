use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const CHROME_MAJOR: &str = "134";
const CHROME_FULL: &str = "134.0.6998.0";
const GREASE_BRAND: &str = "Not-A.Brand";

const NV: &str = "Google Inc. (NVIDIA)";
const AMD: &str = "Google Inc. (AMD)";
const INTEL: &str = "Google Inc. (Intel)";

macro_rules! nv { ($n:expr) => { concat!("ANGLE (NVIDIA, NVIDIA GeForce ", $n, " Direct3D11 vs_5_0 ps_5_0, D3D11)") } }
macro_rules! amd { ($n:expr) => { concat!("ANGLE (AMD, AMD Radeon ", $n, " Direct3D11 vs_5_0 ps_5_0, D3D11)") } }
macro_rules! amd_i { ($n:expr) => { concat!("ANGLE (AMD, AMD Radeon(TM) ", $n, " Direct3D11 vs_5_0 ps_5_0, D3D11)") } }
macro_rules! intel { ($n:expr) => { concat!("ANGLE (Intel, Intel(R) ", $n, " Direct3D11 vs_5_0 ps_5_0, D3D11)") } }

#[derive(Debug, Clone, Copy)]
pub struct HwProfile {
    pub gpu_vendor: &'static str,
    pub gpu_renderer: &'static str,
    pub width: u32,
    pub height: u32,
    pub cores: u32,
    pub memory: u32,
    pub dpr: f64,
    pub max_texture_size: u32,
}

// 100 realistic device profiles — Steam Hardware Survey + realistic CPU/RAM pairings
#[rustfmt::skip]
pub static PROFILES: [HwProfile; 100] = [
    // ── NVIDIA Desktop (25) ──
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 5090"),           width: 3840, height: 2160, cores: 32, memory: 64, dpr: 2.0,  max_texture_size: 32768 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 5090"),           width: 2560, height: 1440, cores: 24, memory: 32, dpr: 1.0,  max_texture_size: 32768 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 5080"),           width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 32768 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 5080"),           width: 3840, height: 2160, cores: 24, memory: 64, dpr: 1.5,  max_texture_size: 32768 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 5070 Ti"),        width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 5070"),           width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4090"),           width: 3840, height: 2160, cores: 24, memory: 64, dpr: 1.5,  max_texture_size: 32768 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4090"),           width: 2560, height: 1440, cores: 32, memory: 32, dpr: 1.0,  max_texture_size: 32768 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4080 SUPER"),     width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4080"),           width: 2560, height: 1440, cores: 24, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4070 Ti SUPER"),  width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4070 Ti"),        width: 1920, height: 1080, cores: 12, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4070 SUPER"),     width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4070"),           width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4060 Ti"),        width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4060"),           width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3090 Ti"),        width: 3840, height: 2160, cores: 24, memory: 64, dpr: 1.5,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3090"),           width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3080 Ti"),        width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3080"),           width: 2560, height: 1440, cores: 12, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3070 Ti"),        width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3070"),           width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3060 Ti"),        width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3060"),           width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3050"),           width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    // ── NVIDIA Older (5) ──
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("GTX 1660 SUPER"),     width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("GTX 1660"),           width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("GTX 1650 SUPER"),     width: 1920, height: 1080, cores:  6, memory:  8, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("GTX 1650"),           width: 1920, height: 1080, cores:  4, memory:  8, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("GTX 1050 Ti"),        width: 1920, height: 1080, cores:  4, memory:  8, dpr: 1.0,  max_texture_size: 16384 },
    // ── AMD Desktop (20) ──
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 9070 XT"),       width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 9070"),          width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7900 XTX"),      width: 3840, height: 2160, cores: 24, memory: 64, dpr: 1.5,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7900 XT"),       width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7900 GRE"),      width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7800 XT"),       width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7700 XT"),       width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7600 XT"),       width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7600"),          width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6950 XT"),       width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6900 XT"),       width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6800 XT"),       width: 2560, height: 1440, cores: 12, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6800"),          width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6700 XT"),       width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6700"),          width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6650 XT"),       width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6600 XT"),       width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6600"),          width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6500 XT"),       width: 1920, height: 1080, cores:  6, memory:  8, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 5700 XT"),       width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    // ── Intel Desktop (10) ──
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Arc(TM) A770 Graphics"),  width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Arc(TM) A750 Graphics"),  width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Arc(TM) A580 Graphics"),  width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics 770"),       width: 1920, height: 1080, cores: 24, memory: 32, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics 770"),       width: 1920, height: 1080, cores: 16, memory: 16, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics 730"),       width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics 630"),       width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics 630"),       width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics 610"),       width: 1920, height: 1080, cores:  4, memory:  8, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("HD Graphics 530"),        width: 1920, height: 1080, cores:  4, memory:  8, dpr: 1.0, max_texture_size: 16384 },
    // ── NVIDIA Laptop (10) ──
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4090 Laptop GPU"),   width: 2560, height: 1600, cores: 16, memory: 32, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4080 Laptop GPU"),   width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4070 Laptop GPU"),   width: 1920, height: 1200, cores: 12, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4060 Laptop GPU"),   width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4050 Laptop GPU"),   width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3080 Laptop GPU"),   width: 2560, height: 1440, cores: 12, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3070 Laptop GPU"),   width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3060 Laptop GPU"),   width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3050 Laptop GPU"),   width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 2060"),              width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    // ── Intel Laptop (7) ──
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Iris(R) Xe Graphics"),    width: 1920, height: 1200, cores: 12, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Iris(R) Xe Graphics"),    width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Iris(R) Xe Graphics"),    width: 2560, height: 1600, cores: 12, memory: 16, dpr: 1.5,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics"),           width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics"),           width: 1920, height: 1080, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Iris(R) Plus Graphics"),  width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("UHD Graphics"),           width: 1920, height: 1080, cores: 16, memory: 32, dpr: 1.25, max_texture_size: 16384 },
    // ── AMD Laptop (8) ──
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd_i!("780M"),                  width: 1920, height: 1200, cores:  8, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd_i!("780M"),                  width: 2560, height: 1600, cores:  8, memory: 16, dpr: 1.5,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd_i!("680M"),                  width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7600M XT"),             width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6700M"),                width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6600M"),                width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd_i!("Vega 8 Graphics"),       width: 1920, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd_i!("Vega 10 Graphics"),      width: 1920, height: 1080, cores:  4, memory:  8, dpr: 1.0,  max_texture_size: 16384 },
    // ── Ultrawide & Alt Configs (15) ──
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4070 Ti"),          width: 3440, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3080"),             width: 3440, height: 1440, cores: 12, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7900 XTX"),       width: 3440, height: 1440, cores: 24, memory: 64, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4090"),             width: 2560, height: 1440, cores: 24, memory: 128, dpr: 1.0, max_texture_size: 32768 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3060"),             width: 1366, height:  768, cores:  6, memory:  8, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4060 Ti"),          width: 2560, height: 1440, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7800 XT"),        width: 2560, height: 1440, cores: 12, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3070 Ti"),          width: 2560, height: 1440, cores: 12, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 7600"),           width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.25, max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 4070 SUPER"),       width: 2560, height: 1440, cores: 16, memory: 32, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 5070 Ti"),          width: 3840, height: 2160, cores: 16, memory: 32, dpr: 1.5,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: INTEL, gpu_renderer: intel!("Arc(TM) A770 Graphics"), width: 1920, height: 1080, cores: 16, memory: 32, dpr: 1.0, max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("RTX 3060"),             width: 2560, height: 1080, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: AMD, gpu_renderer: amd!("RX 6700 XT"),        width: 2560, height: 1440, cores:  8, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
    HwProfile { gpu_vendor: NV, gpu_renderer: nv!("GTX 1660 Ti"),          width: 1920, height: 1080, cores:  6, memory: 16, dpr: 1.0,  max_texture_size: 16384 },
];

static TIMEZONES: &[(&str, &str)] = &[
    ("America/New_York", "en-US"),
    ("America/Chicago", "en-US"),
    ("America/Denver", "en-US"),
    ("America/Los_Angeles", "en-US"),
    ("America/Phoenix", "en-US"),
    ("Europe/London", "en-GB"),
    ("Europe/Paris", "fr-FR"),
    ("Europe/Berlin", "de-DE"),
    ("Europe/Madrid", "es-ES"),
    ("Europe/Rome", "it-IT"),
    ("Europe/Amsterdam", "nl-NL"),
    ("Europe/Warsaw", "pl-PL"),
    ("Europe/Bucharest", "ro-RO"),
    ("Europe/Istanbul", "tr-TR"),
    ("Europe/Moscow", "ru-RU"),
    ("Asia/Tokyo", "ja-JP"),
    ("Asia/Seoul", "ko-KR"),
    ("Asia/Shanghai", "zh-CN"),
    ("Asia/Hong_Kong", "zh-HK"),
    ("Asia/Taipei", "zh-TW"),
    ("Asia/Singapore", "en-SG"),
    ("Asia/Bangkok", "th-TH"),
    ("Australia/Sydney", "en-AU"),
    ("America/Toronto", "en-CA"),
    ("America/Sao_Paulo", "pt-BR"),
    ("Asia/Kolkata", "hi-IN"),
];

static FONTS: &[&str] = &[
    "Arial", "Arial Black", "Calibri", "Cambria", "Candara",
    "Comic Sans MS", "Consolas", "Constantia", "Corbel",
    "Courier New", "Georgia", "Impact", "Lucida Console",
    "Lucida Sans Unicode", "Microsoft Sans Serif", "Palatino Linotype",
    "Segoe Print", "Segoe Script", "Segoe UI", "Segoe UI Symbol",
    "Tahoma", "Times New Roman", "Trebuchet MS", "Verdana",
    "Webdings", "Wingdings",
];

// ── PRNG ──

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn range(&mut self, max: usize) -> usize {
        (self.next() % max as u64) as usize
    }
}

// ── Public API ──

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Pick a random profile index and seed using OS time + counter.
pub fn random_profile() -> (usize, u64) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let seed = nanos ^ count.wrapping_mul(6364136223846793005);
    let idx = (seed as usize) % PROFILES.len();
    (idx, seed)
}

/// Generate a complete clawser config JSON for the given profile and seed.
pub fn generate_config_json(profile_index: usize, seed: u64) -> String {
    let idx = profile_index % PROFILES.len();
    let p = &PROFILES[idx];
    let mut rng = Rng::new(seed);

    let tz_idx = rng.range(TIMEZONES.len());
    let (timezone, locale) = TIMEZONES[tz_idx];

    let langs: Vec<String> = if locale.starts_with("en") {
        vec![locale.to_string(), "en".to_string()]
    } else {
        let base = locale.split('-').next().unwrap();
        vec![locale.to_string(), base.to_string(), "en-US".to_string(), "en".to_string()]
    };

    let config = serde_json::json!({
        "profile_id": format!("clawser_{}_{}", idx, seed),
        "navigator": {
            "user_agent": format!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
                CHROME_MAJOR
            ),
            "app_version": format!(
                "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
                CHROME_MAJOR
            ),
            "platform": "Win32",
            "vendor": "Google Inc.",
            "languages": langs,
            "hardware_concurrency": p.cores,
            "device_memory": p.memory,
            "max_touch_points": 0,
            "user_agent_data": {
                "brands": [
                    { "brand": "Chromium", "version": CHROME_MAJOR },
                    { "brand": "Google Chrome", "version": CHROME_MAJOR },
                    { "brand": GREASE_BRAND, "version": "8" }
                ],
                "mobile": false,
                "platform": "Windows",
                "platform_version": "15.0.0",
                "architecture": "x86",
                "bitness": "64",
                "model": "",
                "full_version_list": [
                    { "brand": "Chromium", "version": CHROME_FULL },
                    { "brand": "Google Chrome", "version": CHROME_FULL },
                    { "brand": GREASE_BRAND, "version": "8.0.0.0" }
                ]
            }
        },
        "screen": {
            "width": p.width,
            "height": p.height,
            "avail_width": p.width,
            "avail_height": p.height.saturating_sub(40),
            "color_depth": 24,
            "pixel_depth": 24,
            "device_pixel_ratio": p.dpr
        },
        "gpu": {
            "vendor": p.gpu_vendor,
            "renderer": p.gpu_renderer
        },
        "noise_seeds": {
            "canvas": rng.next(),
            "webgl": rng.next(),
            "audio": rng.next(),
            "client_rects": rng.next()
        },
        "timezone": timezone,
        "locale": locale,
        "fonts": FONTS,
        "media_devices": {
            "audio_inputs": 1 + rng.range(2),
            "audio_outputs": 1 + rng.range(3),
            "video_inputs": rng.range(2)
        },
        "webrtc": { "policy": "disabled" },
        "battery": false,
        "bluetooth": false,
        "usb": false
    });

    serde_json::to_string_pretty(&config).unwrap()
}

/// Write config JSON to temp file, return the canonical Windows path.
pub fn write_config_file(profile_index: usize, seed: u64) -> std::io::Result<PathBuf> {
    let json = generate_config_json(profile_index, seed);
    let dir = std::env::temp_dir().join("clawser_configs");
    std::fs::create_dir_all(&dir)?;
    let file = dir.join(format!("profile_{}_{}.json", profile_index % PROFILES.len(), seed));
    std::fs::write(&file, &json)?;
    let canonical = file.canonicalize()?;
    // Strip \\?\ UNC prefix on Windows
    let s = canonical.to_string_lossy();
    if let Some(stripped) = s.strip_prefix("\\\\?\\") {
        Ok(PathBuf::from(stripped))
    } else {
        Ok(canonical)
    }
}
