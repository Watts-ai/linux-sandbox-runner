/// Cross-platform Sandbox Runner
///
/// Linux: Uses OpenAI Codex's linux-sandbox (Landlock + seccomp)
/// Windows: Uses OpenAI Codex's windows-sandbox (Restricted tokens + ACLs)
///
/// Both use the exact same CLI interface from codex-linux-sandbox
///
/// For more information, see: https://github.com/openai/codex

#[cfg(target_os = "linux")]
fn main() -> ! {
    codex_linux_sandbox::run_main()
}

#[cfg(target_os = "windows")]
fn main() {
    // Import the same types and parsing that Linux uses
    use clap::Parser;
    use codex_core::protocol::SandboxPolicy;
    use std::path::PathBuf;

    // Use the exact same struct definition as LandlockCommand from linux_run_main.rs
    #[derive(Debug, Parser)]
    struct SandboxCommand {
        #[arg(long = "sandbox-policy-cwd")]
        sandbox_policy_cwd: PathBuf,

        #[arg(long = "sandbox-policy")]
        sandbox_policy: SandboxPolicy,

        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    }

    let args = SandboxCommand::parse();

    if args.command.is_empty() {
        eprintln!("Error: No command specified");
        std::process::exit(1);
    }

    // Get CODEX_HOME or use default
    let codex_home = std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".codex")
        });

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let env_map: std::collections::HashMap<String, String> = std::env::vars().collect();

    // Serialize policy to JSON for Windows sandbox
    let policy_json =
        serde_json::to_string(&args.sandbox_policy).expect("Failed to serialize sandbox policy");

    // Call Windows sandbox
    let result = codex_windows_sandbox::run_windows_sandbox_capture(
        &policy_json,
        &args.sandbox_policy_cwd,
        &codex_home,
        args.command,
        &cwd,
        env_map,
        None, // No timeout
    );

    match result {
        Ok(capture) => std::process::exit(capture.exit_code),
        Err(e) => {
            eprintln!("Sandbox error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn main() {
    eprintln!("Unsupported platform. This binary only works on Linux and Windows.");
    std::process::exit(1);
}
