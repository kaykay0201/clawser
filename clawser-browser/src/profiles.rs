//! 100 realistic device profiles from Steam Hardware Survey + StatCounter data.
//!
//! Each profile represents a real-world device configuration.
//! Profiles are static — zero allocation to access.
//! Multiply by unique seed index for unlimited unique fingerprints.

use std::io::{self, Write as _};

/// A hardware profile matching a real device in the wild.
#[derive(Debug, Clone, Copy)]
pub struct HwProfile {
    pub gl_vendor: &'static str,
    pub gl_renderer: &'static str,
    pub cores: u8,
    pub device_memory: u8, // navigator.deviceMemory (Chrome caps at 8)
    pub screen_width: u16,
    pub screen_height: u16,
}

/// Common timezones weighted by internet user population.
static TIMEZONES: &[&str] = &[
    "America/New_York",
    "America/Chicago",
    "America/Denver",
    "America/Los_Angeles",
    "America/Toronto",
    "America/Sao_Paulo",
    "America/Mexico_City",
    "Europe/London",
    "Europe/Paris",
    "Europe/Berlin",
    "Europe/Madrid",
    "Europe/Rome",
    "Europe/Amsterdam",
    "Europe/Warsaw",
    "Europe/Moscow",
    "Europe/Istanbul",
    "Asia/Tokyo",
    "Asia/Shanghai",
    "Asia/Kolkata",
    "Asia/Singapore",
    "Asia/Seoul",
    "Asia/Dubai",
    "Asia/Bangkok",
    "Asia/Jakarta",
    "Australia/Sydney",
    "Pacific/Auckland",
];

/// Common language configurations.
static LANGUAGES: &[&[&str]] = &[
    &["en-US", "en"],
    &["en-GB", "en"],
    &["en-US"],
    &["de-DE", "de", "en-US", "en"],
    &["fr-FR", "fr", "en-US", "en"],
    &["es-ES", "es", "en-US", "en"],
    &["pt-BR", "pt", "en-US", "en"],
    &["it-IT", "it", "en-US", "en"],
    &["nl-NL", "nl", "en-US", "en"],
    &["pl-PL", "pl", "en-US", "en"],
    &["ja-JP", "ja", "en-US", "en"],
    &["ko-KR", "ko", "en-US", "en"],
    &["zh-CN", "zh", "en-US", "en"],
    &["ru-RU", "ru", "en-US", "en"],
    &["tr-TR", "tr", "en-US", "en"],
];

// 100 profiles: budget_desktop(15) + budget_laptop(15) + midrange_desktop(20)
// + midrange_laptop(15) + highend_desktop(15) + highend_laptop(10) + enthusiast(10)
static PROFILES: &[HwProfile] = &[
    // === BUDGET DESKTOP (15) — office, school, cheap builds ===
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 4, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 730 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 730 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 20, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 4, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Super Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 6GB Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1660 Super Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 6500 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 4, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Arc(TM) A380 Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },

    // === BUDGET LAPTOP (15) — office/student, integrated GPU ===
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 4, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1366, screen_height: 768 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 4, screen_width: 1366, screen_height: 768 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1536, screen_height: 864 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 4, screen_width: 1366, screen_height: 768 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 4, screen_width: 1366, screen_height: 768 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon 760M Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1536, screen_height: 864 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 4, device_memory: 4, screen_width: 1920, screen_height: 1080 },

    // === MIDRANGE DESKTOP (20) — popular gaming/productivity ===
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5060 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 2060 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 6600 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 6600 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7600 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7600 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7600 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (Intel)", gl_renderer: "ANGLE (Intel, Intel(R) Arc(TM) A750 Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },

    // === MIDRANGE LAPTOP (15) — gaming/creator laptops ===
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3050 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3050 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3050 Ti Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4050 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4050 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 6600M Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 12, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7600M XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon 780M Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 8, screen_width: 1920, screen_height: 1080 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 6500M Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 8, device_memory: 8, screen_width: 1920, screen_height: 1080 },

    // === HIGHEND DESKTOP (15) — serious gaming ===
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Super Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 6700 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 6800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7700 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 9070 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },

    // === HIGHEND LAPTOP (10) — gaming laptops ===
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Ti Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 20, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5080 Laptop GPU Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7700S Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 16, device_memory: 8, screen_width: 2560, screen_height: 1440 },

    // === ENTHUSIAST (10) — flagship 4K ===
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Super Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 32, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5080 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5090 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 5090 Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 32, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7900 XT Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7900 XTX Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (AMD)", gl_renderer: "ANGLE (AMD, AMD Radeon RX 7900 XTX Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 32, device_memory: 8, screen_width: 3840, screen_height: 2160 },
    HwProfile { gl_vendor: "Google Inc. (NVIDIA)", gl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Ti Super Direct3D11 vs_5_0 ps_5_0, D3D11)", cores: 24, device_memory: 8, screen_width: 3840, screen_height: 2160 },
];

/// Total number of hardware profiles.
pub const PROFILE_COUNT: usize = 100;

/// Get a hardware profile by index.
pub fn get_profile(index: usize) -> &'static HwProfile {
    &PROFILES[index % PROFILE_COUNT]
}

/// Deterministic seed generation from profile index + seed_index.
/// Same (profile, seed_index) always produces the same fingerprint.
fn mix_seed(profile_idx: u64, seed_idx: u64, salt: u64) -> u64 {
    let mut h = profile_idx.wrapping_mul(0x9E3779B97F4A7C15)
        ^ seed_idx.wrapping_mul(0x6C62272E07BB0142)
        ^ salt;
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D049BB133111EB);
    h ^ (h >> 31)
}

/// Generate a complete clawser config JSON for a given (profile_index, seed_index).
/// Deterministic: same inputs always produce the same config.
pub fn generate_config_json(profile_index: usize, seed_index: u64) -> String {
    let hw = get_profile(profile_index);
    let pi = profile_index as u64;

    let canvas_seed = mix_seed(pi, seed_index, 0xCAFE_BABE_DEAD_BEEF);
    let webgl_seed = mix_seed(pi, seed_index, 0x0123_4567_89AB_CDEF);
    let audio_seed = mix_seed(pi, seed_index, 0xFEDC_BA98_7654_3210);
    let rects_seed = mix_seed(pi, seed_index, 0x1337_C0DE_FACE_B00C);

    let tz_index = mix_seed(pi, seed_index, 0xAAAA) as usize % TIMEZONES.len();
    let lang_index = mix_seed(pi, seed_index, 0xBBBB) as usize % LANGUAGES.len();

    let timezone = TIMEZONES[tz_index];
    let langs = LANGUAGES[lang_index];
    let primary_lang = langs[0];

    // avail_height slightly less than height (taskbar)
    let avail_height = hw.screen_height - 40;

    let langs_json: Vec<String> = langs.iter().map(|l| format!("\"{}\"", l)).collect();
    let langs_str = langs_json.join(", ");

    format!(
        r#"{{
  "version": 1,
  "profile_id": "p{}-s{}",
  "navigator": {{
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36",
    "platform": "Win32",
    "vendor": "Google Inc.",
    "app_version": "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36",
    "language": "{}",
    "languages": [{}],
    "hardware_concurrency": {},
    "device_memory": {},
    "max_touch_points": 0,
    "user_agent_data": {{
      "brands": [
        {{"brand": "Google Chrome", "version": "134"}},
        {{"brand": "Chromium", "version": "134"}},
        {{"brand": "Not-A.Brand", "version": "8"}}
      ],
      "mobile": false,
      "platform": "Windows",
      "platform_version": "15.0.0",
      "architecture": "x86",
      "model": "",
      "bitness": "64"
    }}
  }},
  "screen": {{
    "width": {},
    "height": {},
    "avail_width": {},
    "avail_height": {},
    "color_depth": 24,
    "pixel_depth": 24,
    "device_pixel_ratio": 1.0
  }},
  "gpu": {{
    "vendor": "{}",
    "renderer": "{}"
  }},
  "noise_seeds": {{
    "canvas": {},
    "webgl": {},
    "audio": {},
    "client_rects": {}
  }},
  "timezone": "{}",
  "locale": "{}",
  "media_devices": {{
    "audio_inputs": 1,
    "audio_outputs": 2,
    "video_inputs": 1
  }},
  "webrtc": {{
    "policy": "disabled"
  }},
  "battery": {{
    "enabled": false
  }},
  "bluetooth_enabled": false,
  "usb_enabled": false
}}"#,
        profile_index, seed_index,
        primary_lang, langs_str,
        hw.cores, hw.device_memory,
        hw.screen_width, hw.screen_height,
        hw.screen_width, avail_height,
        hw.gl_vendor, hw.gl_renderer,
        canvas_seed, webgl_seed, audio_seed, rects_seed,
        timezone, primary_lang,
    )
}

/// Write a config to a temp file and return the path.
/// The file is deterministically named so the same profile reuses the same file.
pub fn write_config_file(profile_index: usize, seed_index: u64) -> io::Result<String> {
    let json = generate_config_json(profile_index, seed_index);
    let dir = std::env::temp_dir().join("clawser-profiles");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("p{}-s{}.json", profile_index, seed_index));
    let mut f = std::fs::File::create(&path)?;
    f.write_all(json.as_bytes())?;
    // Use canonicalize to get Windows-style path (not MSYS /tmp/...)
    let canonical = path.canonicalize().unwrap_or(path);
    Ok(canonical.to_string_lossy().to_string())
}

/// Pick a random profile index (0..100) using OS entropy.
pub fn random_profile_index() -> usize {
    let mut buf = [0u8; 8];
    getrandom(&mut buf);
    (u64::from_le_bytes(buf) as usize) % PROFILE_COUNT
}

/// Pick a random seed index using OS entropy.
pub fn random_seed_index() -> u64 {
    let mut buf = [0u8; 8];
    getrandom(&mut buf);
    u64::from_le_bytes(buf)
}

/// Simple OS-level random bytes (no crate needed).
pub fn getrandom(buf: &mut [u8]) {
    #[cfg(windows)]
    {
        #[link(name = "bcrypt")]
        extern "system" {
            fn BCryptGenRandom(
                h: *mut u8,
                pb: *mut u8,
                cb: u32,
                flags: u32,
            ) -> i32;
        }
        unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                0x00000002, // BCRYPT_USE_SYSTEM_PREFERRED_RNG
            );
        }
    }
    #[cfg(not(windows))]
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            let _ = f.read_exact(buf);
        }
    }
}
