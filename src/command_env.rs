use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build a command with the executable search path a terminal-launched Waku
/// normally inherits. Apps opened through LaunchServices only receive the
/// system PATH, which is not enough for script-based CLIs whose shebang uses
/// `/usr/bin/env` (for example, an npm-installed Codex launcher needs `node`).
pub fn command(program: impl AsRef<OsStr>) -> Command {
    let program = program.as_ref();
    #[cfg(windows)]
    let mut command = if matches!(
        Path::new(program)
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("cmd" | "bat")
    ) {
        let mut command = Command::new("cmd.exe");
        command.args(["/d", "/s", "/c"]);
        command.arg(program);
        command
    } else {
        Command::new(program)
    };
    #[cfg(not(windows))]
    let mut command = Command::new(program);
    if let Ok(path) = std::env::join_paths(executable_search_paths()) {
        command.env("PATH", path);
    }
    command
}

pub fn find_executable(name: &str) -> Option<PathBuf> {
    let candidate = Path::new(name);
    if candidate.components().count() > 1 && candidate.is_file() {
        return Some(candidate.to_path_buf());
    }
    executable_search_paths()
        .into_iter()
        .map(|directory| directory.join(name))
        .flat_map(windows_executable_candidates)
        .find(|candidate| candidate.is_file())
}

fn windows_executable_candidates(path: PathBuf) -> impl Iterator<Item = PathBuf> {
    #[cfg(windows)]
    {
        let has_extension = path.extension().is_some();
        if has_extension {
            vec![path].into_iter()
        } else {
            vec![
                path.with_extension("exe"),
                path.with_extension("cmd"),
                path.with_extension("bat"),
            ]
            .into_iter()
        }
    }
    #[cfg(not(windows))]
    {
        std::iter::once(path)
    }
}

/// Resolve a user-supplied binary override: `~` expands to the home
/// directory, a path must point at an existing file, and a bare name searches
/// the same directories as [`find_executable`].
pub fn resolve_binary_override(spec: &str) -> Option<PathBuf> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    if let Some(rest) = spec.strip_prefix("~/") {
        let candidate = dirs::home_dir()?.join(rest);
        return candidate.is_file().then_some(candidate);
    }
    find_executable(spec)
}

pub fn executable_search_path() -> Option<std::ffi::OsString> {
    std::env::join_paths(executable_search_paths()).ok()
}

fn executable_search_paths() -> Vec<PathBuf> {
    search_paths_from(
        std::env::var_os("PATH").as_deref(),
        dirs::home_dir().as_deref(),
    )
}

fn search_paths_from(path: Option<&OsStr>, home: Option<&Path>) -> Vec<PathBuf> {
    let mut directories = path
        .map(|path| std::env::split_paths(path).collect::<Vec<_>>())
        .unwrap_or_default();
    if let Some(home) = home {
        directories.extend([
            home.join(".local/bin"),
            home.join(".bun/bin"),
            home.join(".cargo/bin"),
            home.join(".local/share/mise/shims"),
            home.join(".volta/bin"),
        ]);
    }
    #[cfg(unix)]
    directories.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/sbin"),
    ]);

    let mut seen = HashSet::new();
    directories.retain(|directory| seen.insert(directory.clone()));
    directories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_services_path_is_extended_for_script_based_clis() {
        if cfg!(windows) {
            return;
        }
        let home = Path::new("/Users/example");
        let paths = search_paths_from(Some(OsStr::new("/usr/bin:/bin")), Some(home));

        assert_eq!(paths[0], PathBuf::from("/usr/bin"));
        assert_eq!(paths[1], PathBuf::from("/bin"));
        assert!(paths.contains(&home.join(".bun/bin")));
        assert!(paths.contains(&home.join(".local/share/mise/shims")));
        assert!(paths.contains(&PathBuf::from("/opt/homebrew/bin")));
        assert_eq!(
            paths
                .iter()
                .filter(|path| *path == Path::new("/bin"))
                .count(),
            1
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_command_wraps_cmd_shims_through_cmd_exe() {
        let command = command(r"C:\Users\example\AppData\Roaming\npm\pi.cmd");
        assert_eq!(command.get_program(), "cmd.exe");
        assert_eq!(
            command
                .get_args()
                .map(|argument| argument.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec![
                "/d",
                "/s",
                "/c",
                r"C:\Users\example\AppData\Roaming\npm\pi.cmd"
            ]
        );
    }
}
