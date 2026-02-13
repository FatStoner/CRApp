use std::env;
use std::fs;
use std::process::Command;

/// Perform the update
pub fn perform_update(target_version: Option<String>) -> Result<bool, Box<dyn std::error::Error>> {
    // Clean up old executable first
    cleanup_old_executable()?;

    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner("JustJam-Dev")
        .repo_name("CRApp")
        .bin_name("crap")
        .target(&get_target_triple())
        .current_version(current_version);

    if let Some(v) = target_version {
        builder.target_version_tag(&v);
    }

    // Build the updater
    let status = builder.build()?.update()?;

    match status {
        self_update::Status::UpToDate(v) => {
            println!("Already up to date (version {})", v);
            Ok(false)
        }
        self_update::Status::Updated(v) => {
            println!("Updated to version {}", v);
            Ok(true)
        }
    }
}

/// Check if an update is available without applying it
/// Returns Ok(Some(version)) if update available, Ok(None) if up to date
pub fn check_for_updates() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let current_version = env!("CARGO_PKG_VERSION");
    let target = get_target_triple(); // Capture outside thread

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = self_update::backends::github::Update::configure()
            .repo_owner("JustJam-Dev")
            .repo_name("CRApp")
            .bin_name("crap")
            .target(&target)
            .current_version(current_version)
            .build()
            .and_then(|u| u.get_latest_release());

        let _ = tx.send(result);
    });

    // Wait for result with timeout
    match rx.recv_timeout(std::time::Duration::from_secs(15)) {
        Ok(result) => {
            let updates = result?;
            if self_update::version::bump_is_greater(current_version, &updates.version)? {
                Ok(Some(updates.version))
            } else {
                Ok(None)
            }
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Update check timed out after 15s",
        ))),
        Err(e) => Err(Box::new(e)),
    }
}

/// Get the target triple for the current platform
fn get_target_triple() -> String {
    // For Windows builds, we use x86_64-pc-windows-gnu
    // The self_update crate will look for assets matching this pattern
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "x86_64-pc-windows-gnu".to_string()
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "x86_64-unknown-linux-gnu".to_string()
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin".to_string()
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin".to_string()
    }
}

/// Clean up old executable file if it exists
fn cleanup_old_executable() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;
    let old_exe = current_exe.with_extension("exe.old");

    if old_exe.exists() {
        println!("Cleaning up old executable: {:?}", old_exe);
        match fs::remove_file(&old_exe) {
            Ok(_) => println!("Old executable removed successfully"),
            Err(e) => eprintln!("Warning: Failed to remove old executable: {}", e),
        }
    }

    Ok(())
}

/// Restart the application after an update
#[allow(dead_code)]
pub fn restart_application() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;

    println!("Restarting application: {:?}", current_exe);

    #[cfg(target_os = "windows")]
    {
        // On Windows, spawn the new process and exit immediately
        Command::new(&current_exe).spawn()?;

        // Exit the current process
        std::process::exit(0);
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, use exec to replace the current process
        use std::os::unix::process::CommandExt;
        let err = Command::new(&current_exe).exec();
        // exec only returns if there's an error
        Err(Box::new(err))
    }
}
