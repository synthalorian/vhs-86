use std::backtrace::Backtrace;
use std::fs;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::sync::Once;

use tracing::{error, info};

static CRASH_REPORTER_INIT: Once = Once::new();

/// Initialize the crash reporter by installing a custom panic hook.
/// This captures a backtrace and writes it to a crash report file.
pub fn init() {
    CRASH_REPORTER_INIT.call_once(|| {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let backtrace = Backtrace::capture();
            let report = generate_crash_report(info, &backtrace);
            
            // Write to crash report file
            if let Some(crash_path) = crash_report_path() {
                let _ = fs::create_dir_all(crash_path.parent().unwrap_or(&PathBuf::from(".")));
                if let Ok(mut file) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&crash_path)
                {
                    let _ = writeln!(file, "{}", report);
                }
            }

            // Log the panic via tracing
            error!("Application panicked. Crash report saved.");
            error!("Panic info: {}", info);
            error!("Backtrace:\n{}", backtrace);

            // Also print to stderr for immediate visibility
            eprintln!("\n═══════════════════════════════════════════════════════════════");
            eprintln!("  VHS-86 HAS CRASHED");
            eprintln!("═══════════════════════════════════════════════════════════════");
            eprintln!("{}", report);
            eprintln!("A detailed crash report has been saved.");
            if let Some(path) = crash_report_path() {
                eprintln!("Location: {}", path.display());
            }
            eprintln!("═══════════════════════════════════════════════════════════════\n");

            // Chain to the default hook for standard panic behavior
            default_hook(info);
        }));
        info!("Crash reporter initialized");
    });
}

/// Generate a formatted crash report string.
fn generate_crash_report(info: &panic::PanicHookInfo, backtrace: &Backtrace) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let version = env!("CARGO_PKG_VERSION");
    let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);

    let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic payload".to_string()
    };

    let location = if let Some(loc) = info.location() {
        format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
    } else {
        "unknown location".to_string()
    };

    format!(
        r#"
--- Crash Report ---
Timestamp:   {timestamp}
Version:     v{version}
OS:          {os_info}
Location:    {location}
Payload:     {payload}

Backtrace:
{backtrace}
--- End Crash Report ---
"#
    )
}

/// Returns the path where crash reports are stored.
pub fn crash_report_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("vhs-86").join("crashes.log"))
}

/// Returns the contents of the crash log if it exists.
pub fn read_crash_log() -> Option<String> {
    crash_report_path().and_then(|p| fs::read_to_string(p).ok())
}

/// Clears the crash log file.
pub fn clear_crash_log() -> std::io::Result<()> {
    if let Some(path) = crash_report_path() {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
