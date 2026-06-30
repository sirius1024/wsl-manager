use crate::logger;
use crate::models::{InstanceState, WslInstance, WslVersion};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum WslError {
    #[error("WSL is not installed or not available on this system")]
    NotAvailable,
    #[error("Invalid distribution name: {0}")]
    InvalidName(String),
    #[error("Command '{command}' failed: {message}")]
    CommandFailed { command: String, message: String },
    #[error("Failed to parse WSL output: {0}")]
    ParseError(String),
    #[error("Instance '{0}' not found")]
    InstanceNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

fn mock_mode() -> bool {
    static MOCK: OnceLock<bool> = OnceLock::new();
    *MOCK.get_or_init(|| std::env::var("WSL_MANAGER_MOCK").is_ok())
}

/// Returns the path to `wsl.exe` on Windows.
/// On 32-bit processes, System32 is redirected to SysWOW64, so we fall back to
/// Sysnative to reach the 64-bit `wsl.exe`.
#[cfg(windows)]
fn wsl_program() -> Result<PathBuf, WslError> {
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let system32 = PathBuf::from(&system_root).join("System32").join("wsl.exe");

    if system32.exists() {
        return Ok(system32);
    }

    if cfg!(target_pointer_width = "32") {
        let sysnative = PathBuf::from(&system_root).join("Sysnative").join("wsl.exe");
        if sysnative.exists() {
            return Ok(sysnative);
        }
    }

    Err(WslError::NotAvailable)
}

#[cfg(not(windows))]
fn wsl_program() -> Result<PathBuf, WslError> {
    Err(WslError::NotAvailable)
}

async fn run_wsl(args: &[&str]) -> Result<std::process::Output, WslError> {
    if mock_mode() {
        return Err(WslError::NotAvailable);
    }

    let program = wsl_program()?;
    let mut cmd = Command::new(&program);
    cmd.args(args);

    #[cfg(windows)]
    {
        // CREATE_NO_WINDOW (0x08000000) prevents a flashing console window
        // when WSL is spawned from the GUI backend.
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = decode_output(&output.stderr).trim().to_string();
        let command = format!("wsl {}", args.join(" "));
        return Err(WslError::CommandFailed {
            command,
            message: if stderr.is_empty() {
                "unknown error".to_string()
            } else {
                stderr
            },
        });
    }

    Ok(output)
}

/// WSL command output can be UTF-8, UTF-16 LE/BE, or the system default ANSI
/// code page (e.g. GBK on Chinese Windows). We attempt the most common
/// encodings first to avoid garbled logs.
fn decode_output(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    // UTF-8 BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }

    // UTF-16 LE with BOM
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let u16s: Vec<u16> = bytes[2..]
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
            .collect();
        return String::from_utf16_lossy(&u16s);
    }

    // UTF-16 BE with BOM
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let u16s: Vec<u16> = bytes[2..]
            .chunks(2)
            .map(|c| u16::from_be_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
            .collect();
        return String::from_utf16_lossy(&u16s);
    }

    // Heuristic: UTF-16 text contains many null bytes; UTF-8/ANSI text does not.
    if bytes.contains(&0) {
        let u16s: Vec<u16> = bytes
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
            .collect();
        if let Ok(text) = String::from_utf16(&u16s) {
            return text;
        }
    }

    // Strict UTF-8 first.
    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        return text;
    }

    // Fallback to GBK, the most common non-Unicode code page on Chinese Windows.
    // GBK is a superset of GB2312 and covers Windows code page 936.
    #[cfg(windows)]
    {
        let (text, _, _) = encoding_rs::GBK.decode(bytes);
        return text.into_owned();
    }

    #[cfg(not(windows))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// Validates a WSL distribution name.
/// Allowed characters: letters, digits, dots, hyphens, underscores.
/// Must not be empty and must not start with a hyphen.
fn validate_distro_name(name: &str) -> Result<(), WslError> {
    if name.is_empty() {
        return Err(WslError::InvalidName("name cannot be empty".to_string()));
    }
    if name.starts_with('-') {
        return Err(WslError::InvalidName(format!(
            "name cannot start with '-': {name}"
        )));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        return Err(WslError::InvalidName(format!(
            "name contains invalid characters: {name}"
        )));
    }
    Ok(())
}

pub async fn list_instances() -> Result<Vec<WslInstance>, WslError> {
    if mock_mode() {
        let fixture = include_str!("../fixtures/wsl-list.json");
        return serde_json::from_str(fixture)
            .map_err(|e| WslError::ParseError(format!("mock fixture: {e}")));
    }

    let output = run_wsl(&["--list", "--verbose"]).await?;
    let text = decode_output(&output.stdout);
    logger::log(&format!("wsl --list --verbose output:\n{text}"));

    let mut instances = parse_list_text(&text)?;
    logger::log(&format!("parsed {} instances", instances.len()));

    // Cross-check running instances for systems where STATE is localized.
    match list_running_names().await {
        Ok(running_names) => {
            logger::log(&format!("running names: {:?}", running_names));
            for instance in &mut instances {
                if instance.state == InstanceState::Unknown {
                    instance.state = if running_names.contains(&instance.name) {
                        InstanceState::Running
                    } else {
                        InstanceState::Stopped
                    };
                }
            }
        }
        Err(e) => {
            logger::log(&format!(
                "failed to get running names: {e}. Keeping parsed states."
            ));
        }
    }

    // Fetch distribution and IP details in parallel to keep list refresh fast.
    let mut join_set = tokio::task::JoinSet::new();
    for instance in instances.iter_mut() {
        let name = instance.name.clone();
        let state = instance.state;
        join_set.spawn(async move {
            let distribution = get_distribution(&name).await.ok();
            let ip_address = if state == InstanceState::Running {
                get_ip_address(&name).await.ok()
            } else {
                None
            };
            (name, distribution, ip_address)
        });
    }

    while let Some(result) = join_set.join_next().await {
        if let Ok((name, distribution, ip_address)) = result {
            if let Some(instance) = instances.iter_mut().find(|i| i.name == name) {
                instance.distribution = distribution;
                instance.ip_address = ip_address;
            }
        }
    }

    Ok(instances)
}

pub async fn get_wsl_version() -> Result<WslVersion, WslError> {
    if mock_mode() {
        return Ok(WslVersion {
            wsl_version: Some("2.0.14.0".to_string()),
            kernel_version: Some("5.15.133.1-1".to_string()),
            windows_version: Some("10.0.22631.2861".to_string()),
            fields: std::collections::HashMap::from([
                ("WSL version".to_string(), "2.0.14.0".to_string()),
                ("Kernel version".to_string(), "5.15.133.1-1".to_string()),
                ("Windows version".to_string(), "10.0.22631.2861".to_string()),
            ]),
            raw: "WSL version: 2.0.14.0\nKernel version: 5.15.133.1-1\nWindows version: 10.0.22631.2861".to_string(),
        });
    }

    let mut combined_raw = String::new();
    let mut combined_fields = std::collections::HashMap::new();

    // Newer WSL (Microsoft Store version) supports `wsl --version`.
    match run_wsl(&["--version"]).await {
        Ok(output) => {
            let text = decode_output(&output.stdout);
            logger::log(&format!("wsl --version output:\n{text}"));
            combined_raw.push_str("--version output --\n");
            combined_raw.push_str(&text);
            combined_raw.push('\n');
            for (k, v) in parse_key_value_lines(&text) {
                combined_fields.insert(k, v);
            }
        }
        Err(e) => {
            logger::log(&format!("wsl --version failed: {e}"));
        }
    }

    // Older WSL does not support --version; also use --status for extra fields.
    match run_wsl(&["--status"]).await {
        Ok(output) => {
            let text = decode_output(&output.stdout);
            logger::log(&format!("wsl --status output:\n{text}"));
            combined_raw.push_str("--status output --\n");
            combined_raw.push_str(&text);
            for (k, v) in parse_key_value_lines(&text) {
                // --version values take precedence.
                combined_fields.entry(k).or_insert(v);
            }
        }
        Err(e) => {
            logger::log(&format!("wsl --status failed: {e}"));
        }
    }

    if combined_fields.is_empty() {
        return Err(WslError::NotAvailable);
    }

    let wsl_version = find_field_by_keywords(&combined_fields, &["wsl version", "wsl 版本"]);
    let kernel_version = find_field_by_keywords(&combined_fields, &["kernel version", "kernel 版本", "wslg version"]);
    let windows_version = find_field_by_keywords(&combined_fields, &["windows version", "windows 版本"]);

    Ok(WslVersion {
        wsl_version,
        kernel_version,
        windows_version,
        fields: combined_fields,
        raw: combined_raw.trim().to_string(),
    })
}

fn parse_key_value_lines(text: &str) -> std::collections::HashMap<String, String> {
    let mut fields = std::collections::HashMap::new();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if !key.is_empty() {
                fields.insert(key, value);
            }
        }
    }
    fields
}

fn find_field_by_keywords(
    fields: &std::collections::HashMap<String, String>,
    keywords: &[&str],
) -> Option<String> {
    for keyword in keywords {
        if let Some(value) = fields
            .iter()
            .find(|(k, _)| k.to_lowercase().contains(keyword))
            .map(|(_, v)| v.clone())
        {
            return Some(value);
        }
    }
    None
}

fn parse_list_text(text: &str) -> Result<Vec<WslInstance>, WslError> {
    let mut instances = Vec::new();

    // Try to locate column boundaries from the header row. If the header is
    // localized, fall back to whitespace splitting.
    let (name_start, state_start, version_start) = locate_columns(text);

    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }

        // Detect the default marker. It appears before the NAME column.
        let default = if let Some(start) = name_start {
            let prefix = trimmed.get(..start).unwrap_or("");
            prefix.contains('*')
        } else {
            let content = trimmed.trim_start();
            content.starts_with('*')
        };

        let tokens: Vec<&str> = if let (Some(ns), Some(ss), Some(vs)) =
            (name_start, state_start, version_start)
        {
            let name = substring(trimmed, ns, ss).trim();
            let state = substring(trimmed, ss, vs).trim();
            let version = substring(trimmed, vs, trimmed.len()).trim();
            if name.is_empty() || name.eq_ignore_ascii_case("name") {
                continue;
            }
            vec![name, state, version]
        } else {
            let content = trimmed.trim_start_matches('*').trim_start();
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.is_empty() || parts[0].eq_ignore_ascii_case("name") {
                continue;
            }
            parts
        };

        if tokens.len() < 2 {
            continue;
        }

        let name = tokens[0].to_string();
        let version = match tokens.last().and_then(|s| s.parse().ok()) {
            Some(v) => v,
            // Skip header rows and messages like "no installed distributions".
            None => continue,
        };

        let state = if tokens.len() >= 3 {
            tokens[1].parse().unwrap_or(InstanceState::Unknown)
        } else {
            InstanceState::Unknown
        };

        instances.push(WslInstance {
            name,
            state,
            version,
            default,
            distribution: None,
            ip_address: None,
        });
    }

    Ok(instances)
}

fn locate_columns(text: &str) -> (Option<usize>, Option<usize>, Option<usize>) {
    for line in text.lines() {
        let upper = line.to_ascii_uppercase();
        if let (Some(name_idx), Some(version_idx)) = (upper.find("NAME"), upper.find("VERSION")) {
            // STATE is usually between NAME and VERSION.
            let state_idx = upper.find("STATE").unwrap_or_else(|| {
                // Heuristic midpoint if STATE header is missing/localized.
                name_idx + (version_idx - name_idx) / 2
            });
            return (Some(name_idx), Some(state_idx), Some(version_idx));
        }
    }
    (None, None, None)
}

fn substring(s: &str, start: usize, end: usize) -> &str {
    // locate_columns returns byte indices, so slice directly by byte range.
    let start_byte = start.min(s.len());
    let end_byte = end.max(start_byte).min(s.len());
    &s[start_byte..end_byte]
}

async fn list_running_names() -> Result<HashSet<String>, WslError> {
    let output = run_wsl(&["--list", "--running", "--quiet"]).await?;
    let names: HashSet<String> = decode_output(&output.stdout)
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    Ok(names)
}

async fn get_distribution(name: &str) -> Result<String, WslError> {
    validate_distro_name(name)?;
    let output = run_wsl(&["-d", name, "-e", "cat", "/etc/os-release"]).await?;
    let text = decode_output(&output.stdout);
    parse_os_release_pretty_name(&text)
        .ok_or_else(|| WslError::ParseError("PRETTY_NAME not found in /etc/os-release".to_string()))
}

fn parse_os_release_pretty_name(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            // Remove surrounding quotes if present.
            let value = value.trim();
            let value = value.strip_prefix('"').unwrap_or(value);
            let value = value.strip_suffix('"').unwrap_or(value);
            return Some(value.to_string());
        }
    }
    None
}

async fn get_ip_address(name: &str) -> Result<String, WslError> {
    validate_distro_name(name)?;
    let output = run_wsl(&["-d", name, "-e", "hostname", "-I"]).await?;
    let text = decode_output(&output.stdout);
    text.split_whitespace()
        .next()
        .map(String::from)
        .ok_or_else(|| WslError::ParseError("no IP returned".to_string()))
}

pub async fn start_instance(name: &str) -> Result<(), WslError> {
    validate_distro_name(name)?;
    if mock_mode() {
        return Ok(());
    }

    // Running a no-op command starts the instance without opening a visible
    // terminal window. The instance will remain Running for WSL's idle period.
    let program = wsl_program()?;
    let mut cmd = Command::new(&program);
    cmd.args(["-d", name, "-e", "true"]);

    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }

    cmd.spawn()?;
    Ok(())
}

pub async fn stop_instance(name: &str) -> Result<(), WslError> {
    validate_distro_name(name)?;
    run_wsl(&["--terminate", name]).await?;
    Ok(())
}

pub async fn shutdown() -> Result<(), WslError> {
    run_wsl(&["--shutdown"]).await?;
    Ok(())
}

pub async fn open_terminal(name: &str) -> Result<(), WslError> {
    validate_distro_name(name)?;
    if mock_mode() {
        return Ok(());
    }

    if windows_terminal_available().await {
        let mut cmd = Command::new("wt.exe");
        cmd.args(["wsl.exe", "-d", name]);
        cmd.spawn()?;
        return Ok(());
    }

    // Fallback: open the default console host.
    let program = wsl_program()?;
    Command::new(&program).args(["-d", name]).spawn()?;
    Ok(())
}

async fn windows_terminal_available() -> bool {
    #[cfg(windows)]
    {
        let output = Command::new("cmd")
            .args(["/c", "where", "wt.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .await;
        matches!(output, Ok(out) if out.status.success())
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub async fn set_default_instance(name: &str) -> Result<(), WslError> {
    validate_distro_name(name)?;
    run_wsl(&["--set-default", name]).await?;
    Ok(())
}

pub async fn delete_instance(name: &str) -> Result<(), WslError> {
    validate_distro_name(name)?;
    run_wsl(&["--unregister", name]).await?;
    Ok(())
}

/// List distributions available for installation from the online store.
pub async fn list_online_distributions() -> Result<Vec<String>, WslError> {
    let output = run_wsl(&["--list", "--online"]).await?;
    let text = decode_output(&output.stdout);
    Ok(parse_online_list(&text))
}

fn parse_online_list(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_header = true;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // The header line contains "NAME" and "FRIENDLY NAME".
        if in_header && trimmed.to_ascii_uppercase().contains("NAME") {
            in_header = false;
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if !parts.is_empty() {
            names.push(parts[0].to_string());
        }
    }
    names
}

/// Install a new WSL distribution.
/// `distro` is the distribution name from `wsl --list --online` (e.g. "Ubuntu").
/// `install_name` is the custom instance name; if empty, WSL uses the default name.
/// This spawns `wsl --install -d <distro> --name <install_name> --no-launch`
/// and returns immediately. The actual download/installation is performed by WSL
/// and may trigger UAC.
pub async fn install_distribution(distro: &str, install_name: &str) -> Result<(), WslError> {
    validate_distro_name(distro)?;
    if !install_name.is_empty() {
        validate_distro_name(install_name)?;
    }
    if mock_mode() {
        return Ok(());
    }

    let program = wsl_program()?;
    let mut cmd = Command::new(&program);
    cmd.args(["--install", "-d", distro, "--no-launch"]);
    if !install_name.is_empty() {
        cmd.args(["--name", install_name]);
    }

    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }

    cmd.spawn()?;
    Ok(())
}

/// Rename an existing WSL instance.
/// This is implemented as export + import + unregister, because WSL does not
/// provide a native rename command. The instance data is preserved, but the
/// resulting instance becomes an imported distribution.
pub async fn rename_instance(old_name: &str, new_name: &str) -> Result<(), WslError> {
    validate_distro_name(old_name)?;
    validate_distro_name(new_name)?;

    if mock_mode() {
        return Ok(());
    }

    if old_name == new_name {
        return Err(WslError::InvalidName(
            "new name must be different from old name".to_string(),
        ));
    }

    // Find the old instance to preserve its WSL version and default flag.
    let instances = list_instances().await?;
    let old = instances
        .iter()
        .find(|i| i.name == old_name)
        .ok_or_else(|| WslError::InstanceNotFound(old_name.to_string()))?;
    let version = old.version;
    let was_default = old.default;

    // Make sure the old instance is stopped before export.
    let _ = stop_instance(old_name).await;

    let local_app_data = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let temp_dir = PathBuf::from(&local_app_data).join("wsl-manager-temp");
    tokio::fs::create_dir_all(&temp_dir).await?;

    let temp_tar = temp_dir.join(format!("{old_name}-rename.tar"));
    let install_dir = PathBuf::from(&local_app_data).join("wsl").join(new_name);
    tokio::fs::create_dir_all(&install_dir).await?;

    run_wsl(&[
        "--export",
        old_name,
        temp_tar.to_str().unwrap_or(""),
    ])
    .await?;

    run_wsl(&[
        "--import",
        new_name,
        install_dir.to_str().unwrap_or(""),
        temp_tar.to_str().unwrap_or(""),
        "--version",
        &version.to_string(),
    ])
    .await?;

    if was_default {
        let _ = set_default_instance(new_name).await;
    }

    run_wsl(&["--unregister", old_name]).await?;

    // Best-effort cleanup of the temporary tarball.
    let _ = tokio::fs::remove_file(&temp_tar).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TEXT: &str = r#"  NAME              STATE       VERSION
* Ubuntu            Running     2
  Debian            Stopped     2
  openSUSE-Leap     Stopped     1
"#;

    #[test]
    fn parse_sample_list() {
        let instances = parse_list_text(SAMPLE_TEXT).unwrap();
        assert_eq!(instances.len(), 3);

        assert_eq!(instances[0].name, "Ubuntu");
        assert!(instances[0].default);
        assert_eq!(instances[0].state, InstanceState::Running);
        assert_eq!(instances[0].version, 2);

        assert_eq!(instances[1].name, "Debian");
        assert!(!instances[1].default);
        assert_eq!(instances[1].state, InstanceState::Stopped);
        assert_eq!(instances[1].version, 2);

        assert_eq!(instances[2].name, "openSUSE-Leap");
        assert_eq!(instances[2].version, 1);
    }

    #[test]
    fn parse_localized_header_fallback() {
        let text = "  名称      状态       版本\n* Ubuntu    正在运行     2\n  Debian    已停止       2\n";
        let instances = parse_list_text(text).unwrap();
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].name, "Ubuntu");
        assert!(instances[0].default);
        assert_eq!(instances[1].name, "Debian");
    }

    #[test]
    fn parse_empty_list() {
        let instances = parse_list_text("Windows Subsystem for Linux has no installed distributions.").unwrap();
        assert!(instances.is_empty());
    }

    #[test]
    fn validate_name_rejects_invalid() {
        assert!(validate_distro_name("").is_err());
        assert!(validate_distro_name("--flag").is_err());
        assert!(validate_distro_name("Ubuntu 22.04").is_err());
        assert!(validate_distro_name("Ubuntu;rm").is_err());
        assert!(validate_distro_name("Ubuntu-22.04").is_ok());
        assert!(validate_distro_name("docker-desktop").is_ok());
    }

    #[test]
    fn parse_online_list_sample() {
        let text = "NAME                                   FRIENDLY NAME\nUbuntu                                 Ubuntu\nDebian                                 Debian GNU/Linux\n";
        let names = parse_online_list(text);
        assert_eq!(names, vec!["Ubuntu", "Debian"]);
    }

    #[tokio::test]
    async fn mock_fixture_loads() {
        std::env::set_var("WSL_MANAGER_MOCK", "1");
        let instances = list_instances().await.unwrap();
        assert!(!instances.is_empty());
        std::env::remove_var("WSL_MANAGER_MOCK");
    }
}
