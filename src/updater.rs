use std::env;
use std::fs;
use std::process::Command;

/// Check for updates and perform the update if available
#[allow(dead_code)]
pub fn check_and_update() -> Result<bool, Box<dyn std::error::Error>> {
    // Clean up old executable first
    cleanup_old_executable()?;

    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    // Build the updater
    let status = self_update::backends::github::Update::configure()
        .repo_owner("JustJam-Dev")
        .repo_name("CRApp")
        .bin_name("crap")
        .target(&get_target_triple())
        .current_version(current_version)
        .build()?
        .update()?;

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
