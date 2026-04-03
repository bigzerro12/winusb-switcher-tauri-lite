#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Linux elevation helpers (single PolicyKit session per invocation):
    // - `pkexec <this_exe> --lite-extract-jlink <zip_path> <dst_dir>` — extract to /opt, install udev rules from the bundle, chmod +x.
    // - `pkexec <this_exe> --lite-install-udev <rules_file>` — copy rules to /etc/udev when extraction did not need root.
    #[cfg(target_os = "linux")]
    {
        let mut args = std::env::args().skip(1);
        if let Some(flag) = args.next() {
            if flag == "--lite-extract-jlink" {
                let zip_path = args.next().unwrap_or_default();
                let dst_dir = args.next().unwrap_or_default();
                if zip_path.is_empty() || dst_dir.is_empty() {
                    eprintln!("Usage: --lite-extract-jlink <zip_path> <dst_dir>");
                    std::process::exit(2);
                }
                let zip_path = std::path::PathBuf::from(zip_path);
                let dst_dir = std::path::PathBuf::from(dst_dir);
                match winusb_switcher_lite_lib::extract_zip(&zip_path, &dst_dir).and_then(|_| {
                    winusb_switcher_lite_lib::linux_try_install_segger_udev_after_extract(&dst_dir)
                })
                .and_then(|_| winusb_switcher_lite_lib::linux_post_extract_fixups(&dst_dir)) {
                    Ok(()) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Extraction failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if flag == "--lite-install-udev" {
                let rules = args.next().unwrap_or_default();
                if rules.is_empty() {
                    eprintln!("Usage: --lite-install-udev <path_to_jlink.rules>");
                    std::process::exit(2);
                }
                let rules = std::path::PathBuf::from(rules);
                match winusb_switcher_lite_lib::linux_install_segger_udev_rules_from_src(&rules) {
                    Ok(()) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("udev install failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        let _ = ctrlc::set_handler(|| std::process::exit(0));
    }
    winusb_switcher_lite_lib::run();
}