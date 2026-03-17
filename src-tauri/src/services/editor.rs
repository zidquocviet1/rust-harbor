use std::process::Command;
use std::path::Path;
use crate::models::editor::EditorInfo;

#[derive(Debug, Clone)]
struct SupportedEditor {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    macos_bundle_id: Option<&'static str>,
    macos_app_name: Option<&'static str>, // Fallback: app name in /Applications
    windows_app_path: Option<&'static str>,
    linux_bin: Option<&'static str>,
    binary_name: &'static str,
}

const SUPPORTED_EDITORS: &[SupportedEditor] = &[
    SupportedEditor {
        id: "vscode",
        name: "Visual Studio Code",
        icon: "visualstudiocode",
        macos_bundle_id: Some("com.microsoft.VSCode"),
        macos_app_name: Some("Visual Studio Code.app"),
        windows_app_path: Some("Microsoft VS Code/bin/code.cmd"),
        linux_bin: Some("code"),
        binary_name: "code",
    },
    SupportedEditor {
        id: "vscode-insiders",
        name: "VS Code Insiders",
        icon: "visualstudiocode",
        macos_bundle_id: Some("com.microsoft.VSCodeInsiders"),
        macos_app_name: Some("Visual Studio Code - Insiders.app"),
        windows_app_path: Some("Microsoft VS Code Insiders/bin/code-insiders.cmd"),
        linux_bin: Some("code-insiders"),
        binary_name: "code-insiders",
    },
    SupportedEditor {
        id: "cursor",
        name: "Cursor",
        icon: "cursor",
        macos_bundle_id: None, // Cursor bundle ID varies between versions
        macos_app_name: Some("Cursor.app"),
        windows_app_path: Some("Cursor/bin/cursor.cmd"),
        linux_bin: Some("cursor"),
        binary_name: "cursor",
    },
    SupportedEditor {
        id: "zed",
        name: "Zed",
        icon: "zed",
        macos_bundle_id: Some("dev.zed.Zed"),
        macos_app_name: Some("Zed.app"),
        windows_app_path: None,
        linux_bin: Some("zed"),
        binary_name: "zed",
    },
    SupportedEditor {
        id: "sublime-text",
        name: "Sublime Text",
        icon: "sublimetext",
        macos_bundle_id: Some("com.sublimetext.4"),
        macos_app_name: Some("Sublime Text.app"),
        windows_app_path: Some("Sublime Text/subl.exe"),
        linux_bin: Some("subl"),
        binary_name: "subl",
    },
    SupportedEditor {
        id: "intellij-idea",
        name: "IntelliJ IDEA",
        icon: "intellijidea",
        macos_bundle_id: Some("com.jetbrains.intellij"),
        macos_app_name: None, // IntelliJ has version-specific names
        windows_app_path: Some("JetBrains/IntelliJ IDEA/bin/idea64.exe"),
        linux_bin: Some("idea"),
        binary_name: "idea",
    },
    SupportedEditor {
        id: "webstorm",
        name: "WebStorm",
        icon: "webstorm",
        macos_bundle_id: Some("com.jetbrains.WebStorm"),
        macos_app_name: None,
        windows_app_path: Some("JetBrains/WebStorm/bin/webstorm64.exe"),
        linux_bin: Some("webstorm"),
        binary_name: "webstorm",
    },
    SupportedEditor {
        id: "pycharm",
        name: "PyCharm",
        icon: "pycharm",
        macos_bundle_id: Some("com.jetbrains.pycharm"),
        macos_app_name: None,
        windows_app_path: Some("JetBrains/PyCharm/bin/pycharm64.exe"),
        linux_bin: Some("pycharm"),
        binary_name: "pycharm",
    },
];

pub fn get_installed_editors() -> Vec<EditorInfo> {
    let mut installed = Vec::new();

    for editor in SUPPORTED_EDITORS {
        if is_editor_installed(editor) {
            installed.push(EditorInfo {
                id: editor.id.to_string(),
                name: editor.name.to_string(),
                icon: editor.icon.to_string(),
            });
        }
    }

    installed
}

fn is_editor_installed(editor: &SupportedEditor) -> bool {
    #[cfg(target_os = "macos")]
    {
        // First, check via bundle ID using mdfind (most reliable)
        if let Some(bundle_id) = editor.macos_bundle_id {
            let output = Command::new("mdfind")
                .arg(format!("kMDItemCFBundleIdentifier == '{}'", bundle_id))
                .output();

            if let Ok(output) = output {
                if !output.stdout.is_empty() {
                    return true;
                }
            }
        }

        // Fallback: check /Applications directly for known app names
        if let Some(app_name) = editor.macos_app_name {
            let app_path = format!("/Applications/{}", app_name);
            if Path::new(&app_path).exists() {
                return true;
            }
            // Also check user's Applications folder
            if let Ok(home) = std::env::var("HOME") {
                let user_app_path = format!("{}/Applications/{}", home, app_name);
                if Path::new(&user_app_path).exists() {
                    return true;
                }
            }
        }

        // For JetBrains IDEs, check for any version in /Applications
        if editor.id.starts_with("intellij") || editor.id == "webstorm" || editor.id == "pycharm" {
            if let Ok(entries) = std::fs::read_dir("/Applications") {
                let search_name = match editor.id {
                    "intellij-idea" => "IntelliJ IDEA",
                    "webstorm" => "WebStorm",
                    "pycharm" => "PyCharm",
                    _ => "",
                };
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains(search_name) && name.ends_with(".app") {
                        return true;
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try to find the binary in common locations or via 'where'
        if Command::new("where").arg(editor.binary_name).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // Check LOCALAPPDATA or PROGRAMFILES if windows_app_path is defined
        if let Some(app_path) = editor.windows_app_path {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let program_files = std::env::var("ProgramFiles").unwrap_or_default();

            let paths = [
                format!("{}/{}", local_app_data, app_path),
                format!("{}/{}", program_files, app_path),
            ];

            for path in paths {
                if Path::new(&path).exists() {
                    return true;
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(bin) = editor.linux_bin {
            if Command::new("which").arg(bin).status().map(|s| s.success()).unwrap_or(false) {
                return true;
            }
        }
    }

    false
}

pub fn open_path_in_editor(editor_id: &str, path: &str) -> Result<(), String> {
    let editor = SUPPORTED_EDITORS.iter().find(|e| e.id == editor_id)
        .ok_or_else(|| format!("Editor '{}' not supported", editor_id))?;

    #[cfg(target_os = "macos")]
    {
        // Try bundle ID first
        if let Some(bundle_id) = editor.macos_bundle_id {
            let status = Command::new("open")
                .arg("-b")
                .arg(bundle_id)
                .arg(path)
                .status()
                .map_err(|e| e.to_string())?;

            if status.success() {
                return Ok(());
            }
        }

        // Fallback: use app name directly
        if let Some(app_name) = editor.macos_app_name {
            let app_path = format!("/Applications/{}", app_name);
            if Path::new(&app_path).exists() {
                let status = Command::new("open")
                    .arg("-a")
                    .arg(&app_path)
                    .arg(path)
                    .status()
                    .map_err(|e| e.to_string())?;

                if status.success() {
                    return Ok(());
                }
            }

            // Check user's Applications folder
            if let Ok(home) = std::env::var("HOME") {
                let user_app_path = format!("{}/Applications/{}", home, app_name);
                if Path::new(&user_app_path).exists() {
                    let status = Command::new("open")
                        .arg("-a")
                        .arg(&user_app_path)
                        .arg(path)
                        .status()
                        .map_err(|e| e.to_string())?;

                    if status.success() {
                        return Ok(());
                    }
                }
            }
        }

        // For JetBrains IDEs, find the actual app
        if editor.id.starts_with("intellij") || editor.id == "webstorm" || editor.id == "pycharm" {
            if let Ok(entries) = std::fs::read_dir("/Applications") {
                let search_name = match editor.id {
                    "intellij-idea" => "IntelliJ IDEA",
                    "webstorm" => "WebStorm",
                    "pycharm" => "PyCharm",
                    _ => "",
                };
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains(search_name) && name.ends_with(".app") {
                        let app_path = format!("/Applications/{}", name);
                        let status = Command::new("open")
                            .arg("-a")
                            .arg(&app_path)
                            .arg(path)
                            .status()
                            .map_err(|e| e.to_string())?;

                        if status.success() {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try running the binary name directly if it's in PATH
        let status = Command::new("cmd")
            .arg("/c")
            .arg(editor.binary_name)
            .arg(path)
            .status();

        if status.is_ok() && status.unwrap().success() {
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(bin) = editor.linux_bin {
            let status = Command::new(bin)
                .arg(path)
                .status()
                .map_err(|e| e.to_string())?;

            if status.success() {
                return Ok(());
            }
        }
    }

    Err(format!("Failed to open path in {}", editor.name))
}
