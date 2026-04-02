//! Probe scanning and firmware detection.

use crate::error::AppResult;
use crate::jlink::{runner, scripts, types::Probe};

pub fn scan_probes(bin: &str) -> AppResult<Vec<Probe>> {
    log::info!("[jlink] Scanning for probes...");

    let (stdout, _) = runner::run(bin, scripts::show_emu_list())?;
    let mut probes = parse_probe_list(&stdout);

    log::info!("[jlink] Found {} probes", probes.len());

    if !probes.is_empty() {
        log::info!("[jlink] Fetching firmware dates for {} probe(s)...", probes.len());
        let firmware = fetch_firmware_dates(bin, probes.len());
        log::info!("[jlink] Firmware fetch complete");
        for (i, probe) in probes.iter_mut().enumerate() {
            probe.firmware = firmware.get(i).and_then(|f| f.clone());
        }
    }

    log::info!("[jlink] scan_probes complete — returning {} probe(s)", probes.len());
    Ok(probes)
}

fn parse_probe_list(stdout: &str) -> Vec<Probe> {
    let mut probes = Vec::new();
    for line in stdout.lines() {
        let line = if let Some(pos) = line.find("J-Link[") { &line[pos..] } else { continue };
        let serial = extract_field(line, "Serial number: ");
        if serial.is_empty() { continue; }
        let nickname_raw = extract_field(line, "Nickname: ");
        let nickname = if nickname_raw == "<not set>" { String::new() } else { nickname_raw };
        probes.push(Probe {
            id: serial.clone(),
            serial_number: serial,
            product_name: extract_field(line, "ProductName: "),
            nick_name: nickname,
            provider: "JLink".to_string(),
            connection: extract_field(line, "Connection: "),
            driver: "Unknown".to_string(),
            firmware: None,
        });
    }
    probes
}

fn extract_field(line: &str, prefix: &str) -> String {
    if let Some(pos) = line.find(prefix) {
        let rest = &line[pos + prefix.len()..];
        let end = rest.find(',').unwrap_or(rest.len());
        rest[..end].trim().to_string()
    } else {
        String::new()
    }
}

fn fetch_firmware_dates(bin: &str, count: usize) -> Vec<Option<String>> {
    let input = scripts::fetch_firmware_dates(count);

    let (stdout, _) = match runner::run(bin, &input) {
        Ok(r) => r,
        Err(e) => {
            log::error!("[jlink] firmware fetch failed: {}", e);
            return vec![None; count];
        }
    };

    // Collect `Firmware: ... compiled <date>` lines in order. This is more reliable than
    // splitting on "Connecting...O.K." because on Linux the banner often shows
    // `Connecting to J-Link via USB...FAILED` (or mixed OK/FAIL), which breaks delimiter-based parsing.
    let mut results = vec![None; count];
    let mut idx = 0usize;
    for line in stdout.lines() {
        if line.contains("Firmware:") && line.contains("compiled") {
            if let Some(pos) = line.find("compiled ") {
                let date = line[pos + 9..].trim().to_string();
                if !date.is_empty() && idx < count {
                    log::info!("[jlink] Probe[{}] firmware: {}", idx, date);
                    results[idx] = Some(date);
                    idx += 1;
                }
            }
        }
    }
    if idx == 0 && count > 0 {
        log::warn!(
            "[jlink] No Firmware: lines parsed (USB connect may be flaky; check udev/plugdev and close other J-Link tools)"
        );
    }
    results
}