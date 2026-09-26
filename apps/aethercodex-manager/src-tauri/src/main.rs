#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    // Headless entry-point management, used by the Linux tarball installer and
    // by anyone scripting an install without a desktop session.
    if let Some(exit_code) = run_entrypoint_command(&args) {
        std::process::exit(exit_code);
    }
    if args.iter().any(|arg| arg == "--show-update") {
        unsafe {
            std::env::set_var("AETHERCODEX_SHOW_UPDATE", "1");
        }
    }
    aethercodex_manager_lib::run();
}

fn run_entrypoint_command(args: &[String]) -> Option<i32> {
    let install = args.iter().any(|arg| arg == "--install-entrypoints");
    let uninstall = args.iter().any(|arg| arg == "--uninstall-entrypoints");
    if install && uninstall {
        eprintln!("--install-entrypoints 与 --uninstall-entrypoints 不能同时使用");
        return Some(2);
    }
    let result = if install {
        aethercodex_manager_lib::install::install_entrypoints()
    } else if uninstall {
        aethercodex_manager_lib::install::uninstall_entrypoints(Default::default())
    } else {
        return None;
    };
    if result.status == "ok" {
        println!("{}", result.message);
        Some(0)
    } else {
        eprintln!("{}", result.message);
        Some(1)
    }
}
