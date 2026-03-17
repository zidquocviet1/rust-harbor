use std::process::Command;
use std::path::Path;
use crate::models::editor::EditorInfo;

#[derive(Debug, Clone)]
struct SupportedEditor {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    macos_bundle_id: Option<&'static str>,
    macos_app_name: Option<&'static str>,
    /// Partial name to scan for in /Applications (for version-specific app names)
    macos_scan_name: Option<&'static str>,
    windows_app_path: Option<&'static str>,
    linux_bin: Option<&'static str>,
    binary_name: &'static str,
    /// Terminal-only editors (vim, neovim, etc.) — opened via Terminal.app on macOS
    is_terminal_editor: bool,
}

const SUPPORTED_EDITORS: &[SupportedEditor] = &[
    // ── VS Code family ────────────────────────────────────────────────────────
    SupportedEditor {
        id: "vscode",
        name: "Visual Studio Code",
        icon: "visualstudiocode",
        macos_bundle_id: Some("com.microsoft.VSCode"),
        macos_app_name: Some("Visual Studio Code.app"),
        macos_scan_name: None,
        windows_app_path: Some("Microsoft VS Code/bin/code.cmd"),
        linux_bin: Some("code"),
        binary_name: "code",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "vscode-insiders",
        name: "VS Code Insiders",
        icon: "visualstudiocode",
        macos_bundle_id: Some("com.microsoft.VSCodeInsiders"),
        macos_app_name: Some("Visual Studio Code - Insiders.app"),
        macos_scan_name: None,
        windows_app_path: Some("Microsoft VS Code Insiders/bin/code-insiders.cmd"),
        linux_bin: Some("code-insiders"),
        binary_name: "code-insiders",
        is_terminal_editor: false,
    },
    // ── AI-powered editors ────────────────────────────────────────────────────
    SupportedEditor {
        id: "cursor",
        name: "Cursor",
        icon: "cursor",
        macos_bundle_id: None,
        macos_app_name: Some("Cursor.app"),
        macos_scan_name: None,
        windows_app_path: Some("Cursor/bin/cursor.cmd"),
        linux_bin: Some("cursor"),
        binary_name: "cursor",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "windsurf",
        name: "Windsurf",
        icon: "windsurf",
        macos_bundle_id: Some("com.codeium.windsurf"),
        macos_app_name: Some("Windsurf.app"),
        macos_scan_name: None,
        windows_app_path: Some("Windsurf/bin/windsurf.cmd"),
        linux_bin: Some("windsurf"),
        binary_name: "windsurf",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "antigravity",
        name: "Antigravity",
        icon: "antigravity",
        macos_bundle_id: Some("com.google.antigravity"),
        macos_app_name: Some("Antigravity.app"),
        macos_scan_name: None,
        windows_app_path: Some("Antigravity/bin/antigravity.cmd"),
        linux_bin: Some("antigravity"),
        binary_name: "antigravity",
        is_terminal_editor: false,
    },
    // ── Other general editors ─────────────────────────────────────────────────
    SupportedEditor {
        id: "zed",
        name: "Zed",
        icon: "zedindustries",
        macos_bundle_id: Some("dev.zed.Zed"),
        macos_app_name: Some("Zed.app"),
        macos_scan_name: None,
        windows_app_path: None,
        linux_bin: Some("zed"),
        binary_name: "zed",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "sublime-text",
        name: "Sublime Text",
        icon: "sublimetext",
        macos_bundle_id: Some("com.sublimetext.4"),
        macos_app_name: Some("Sublime Text.app"),
        macos_scan_name: None,
        windows_app_path: Some("Sublime Text/subl.exe"),
        linux_bin: Some("subl"),
        binary_name: "subl",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "nova",
        name: "Nova",
        icon: "nova",
        macos_bundle_id: Some("com.panic.Nova"),
        macos_app_name: Some("Nova.app"),
        macos_scan_name: None,
        windows_app_path: None,
        linux_bin: None,
        binary_name: "nova",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "textmate",
        name: "TextMate",
        icon: "textmate",
        macos_bundle_id: Some("com.macromates.TextMate"),
        macos_app_name: Some("TextMate.app"),
        macos_scan_name: None,
        windows_app_path: None,
        linux_bin: None,
        binary_name: "mate",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "bbedit",
        name: "BBEdit",
        icon: "bbedit",
        macos_bundle_id: Some("com.barebones.bbedit"),
        macos_app_name: Some("BBEdit.app"),
        macos_scan_name: None,
        windows_app_path: None,
        linux_bin: None,
        binary_name: "bbedit",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "lapce",
        name: "Lapce",
        icon: "lapce",
        macos_bundle_id: None,
        macos_app_name: Some("Lapce.app"),
        macos_scan_name: None,
        windows_app_path: Some("Lapce/lapce.exe"),
        linux_bin: Some("lapce"),
        binary_name: "lapce",
        is_terminal_editor: false,
    },
    // ── Platform-specific IDEs ────────────────────────────────────────────────
    SupportedEditor {
        id: "xcode",
        name: "Xcode",
        icon: "xcode",
        macos_bundle_id: Some("com.apple.dt.Xcode"),
        macos_app_name: Some("Xcode.app"),
        macos_scan_name: None,
        windows_app_path: None,
        linux_bin: None,
        binary_name: "xcode",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "android-studio",
        name: "Android Studio",
        icon: "androidstudio",
        macos_bundle_id: Some("com.google.android.studio"),
        macos_app_name: None,
        macos_scan_name: Some("Android Studio"),
        windows_app_path: Some("Android/Android Studio/bin/studio64.exe"),
        linux_bin: Some("android-studio"),
        binary_name: "studio",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "notepad-plus-plus",
        name: "Notepad++",
        icon: "notepadplusplus",
        macos_bundle_id: None,
        macos_app_name: None,
        macos_scan_name: None,
        windows_app_path: Some("Notepad++/notepad++.exe"),
        linux_bin: None,
        binary_name: "notepad++",
        is_terminal_editor: false,
    },
    // ── JetBrains family ──────────────────────────────────────────────────────
    SupportedEditor {
        id: "intellij-idea",
        name: "IntelliJ IDEA",
        icon: "intellijidea",
        macos_bundle_id: Some("com.jetbrains.intellij"),
        macos_app_name: None,
        macos_scan_name: Some("IntelliJ IDEA"),
        windows_app_path: Some("JetBrains/IntelliJ IDEA/bin/idea64.exe"),
        linux_bin: Some("idea"),
        binary_name: "idea",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "webstorm",
        name: "WebStorm",
        icon: "webstorm",
        macos_bundle_id: Some("com.jetbrains.WebStorm"),
        macos_app_name: None,
        macos_scan_name: Some("WebStorm"),
        windows_app_path: Some("JetBrains/WebStorm/bin/webstorm64.exe"),
        linux_bin: Some("webstorm"),
        binary_name: "webstorm",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "pycharm",
        name: "PyCharm",
        icon: "pycharm",
        macos_bundle_id: Some("com.jetbrains.pycharm"),
        macos_app_name: None,
        macos_scan_name: Some("PyCharm"),
        windows_app_path: Some("JetBrains/PyCharm/bin/pycharm64.exe"),
        linux_bin: Some("pycharm"),
        binary_name: "pycharm",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "clion",
        name: "CLion",
        icon: "clion",
        macos_bundle_id: Some("com.jetbrains.clion"),
        macos_app_name: None,
        macos_scan_name: Some("CLion"),
        windows_app_path: Some("JetBrains/CLion/bin/clion64.exe"),
        linux_bin: Some("clion"),
        binary_name: "clion",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "goland",
        name: "GoLand",
        icon: "goland",
        macos_bundle_id: Some("com.jetbrains.goland"),
        macos_app_name: None,
        macos_scan_name: Some("GoLand"),
        windows_app_path: Some("JetBrains/GoLand/bin/goland64.exe"),
        linux_bin: Some("goland"),
        binary_name: "goland",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "rider",
        name: "Rider",
        icon: "rider",
        macos_bundle_id: Some("com.jetbrains.rider"),
        macos_app_name: None,
        macos_scan_name: Some("Rider"),
        windows_app_path: Some("JetBrains/Rider/bin/rider64.exe"),
        linux_bin: Some("rider"),
        binary_name: "rider",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "rubymine",
        name: "RubyMine",
        icon: "rubymine",
        macos_bundle_id: Some("com.jetbrains.rubymine"),
        macos_app_name: None,
        macos_scan_name: Some("RubyMine"),
        windows_app_path: Some("JetBrains/RubyMine/bin/rubymine64.exe"),
        linux_bin: Some("rubymine"),
        binary_name: "rubymine",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "phpstorm",
        name: "PhpStorm",
        icon: "phpstorm",
        macos_bundle_id: Some("com.jetbrains.phpstorm"),
        macos_app_name: None,
        macos_scan_name: Some("PhpStorm"),
        windows_app_path: Some("JetBrains/PhpStorm/bin/phpstorm64.exe"),
        linux_bin: Some("phpstorm"),
        binary_name: "phpstorm",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "datagrip",
        name: "DataGrip",
        icon: "datagrip",
        macos_bundle_id: Some("com.jetbrains.datagrip"),
        macos_app_name: None,
        macos_scan_name: Some("DataGrip"),
        windows_app_path: Some("JetBrains/DataGrip/bin/datagrip64.exe"),
        linux_bin: Some("datagrip"),
        binary_name: "datagrip",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "dataspell",
        name: "DataSpell",
        icon: "dataspell",
        macos_bundle_id: Some("com.jetbrains.dataspell"),
        macos_app_name: None,
        macos_scan_name: Some("DataSpell"),
        windows_app_path: Some("JetBrains/DataSpell/bin/dataspell64.exe"),
        linux_bin: Some("dataspell"),
        binary_name: "dataspell",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "rustrover",
        name: "RustRover",
        icon: "rustrover",
        macos_bundle_id: Some("com.jetbrains.rustrover"),
        macos_app_name: None,
        macos_scan_name: Some("RustRover"),
        windows_app_path: Some("JetBrains/RustRover/bin/rustrover64.exe"),
        linux_bin: Some("rustrover"),
        binary_name: "rustrover",
        is_terminal_editor: false,
    },
    SupportedEditor {
        id: "fleet",
        name: "Fleet",
        icon: "fleet",
        macos_bundle_id: Some("com.jetbrains.fleet"),
        macos_app_name: Some("Fleet.app"),
        macos_scan_name: None,
        windows_app_path: Some("JetBrains/Fleet/bin/fleet.exe"),
        linux_bin: Some("fleet"),
        binary_name: "fleet",
        is_terminal_editor: false,
    },
    // ── Terminal editors ──────────────────────────────────────────────────────
    SupportedEditor {
        id: "neovim",
        name: "Neovim",
        icon: "neovim",
        macos_bundle_id: None,
        macos_app_name: None,
        macos_scan_name: None,
        windows_app_path: Some("Neovim/bin/nvim.exe"),
        linux_bin: Some("nvim"),
        binary_name: "nvim",
        is_terminal_editor: true,
    },
    SupportedEditor {
        id: "vim",
        name: "Vim",
        icon: "vim",
        macos_bundle_id: None,
        macos_app_name: None,
        macos_scan_name: None,
        windows_app_path: Some("Git/usr/bin/vim.exe"),
        linux_bin: Some("vim"),
        binary_name: "vim",
        is_terminal_editor: true,
    },
    SupportedEditor {
        id: "emacs",
        name: "Emacs",
        icon: "gnuemacs",
        macos_bundle_id: None,
        macos_app_name: Some("Emacs.app"),
        macos_scan_name: None,
        windows_app_path: Some("GNU Emacs/bin/emacs.exe"),
        linux_bin: Some("emacs"),
        binary_name: "emacs",
        is_terminal_editor: true,
    },
    SupportedEditor {
        id: "helix",
        name: "Helix",
        icon: "helix",
        macos_bundle_id: None,
        macos_app_name: None,
        macos_scan_name: None,
        windows_app_path: Some("Helix/hx.exe"),
        linux_bin: Some("hx"),
        binary_name: "hx",
        is_terminal_editor: true,
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
        // 1. Check via bundle ID using mdfind (most reliable)
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

        // 2. Check exact app name in /Applications
        if let Some(app_name) = editor.macos_app_name {
            let app_path = format!("/Applications/{}", app_name);
            if Path::new(&app_path).exists() {
                return true;
            }
            if let Ok(home) = std::env::var("HOME") {
                let user_app_path = format!("{}/Applications/{}", home, app_name);
                if Path::new(&user_app_path).exists() {
                    return true;
                }
            }
        }

        // 3. Scan /Applications for version-specific app names (JetBrains, Android Studio, etc.)
        if let Some(scan_name) = editor.macos_scan_name {
            if let Ok(entries) = std::fs::read_dir("/Applications") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains(scan_name) && name.ends_with(".app") {
                        return true;
                    }
                }
            }
            if let Ok(home) = std::env::var("HOME") {
                if let Ok(entries) = std::fs::read_dir(format!("{}/Applications", home)) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.contains(scan_name) && name.ends_with(".app") {
                            return true;
                        }
                    }
                }
            }
        }

        // 4. For terminal editors: check if the binary is in PATH
        if editor.is_terminal_editor {
            if Command::new("which")
                .arg(editor.binary_name)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return true;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if Command::new("where")
            .arg(editor.binary_name)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        if let Some(app_path) = editor.windows_app_path {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let program_files = std::env::var("ProgramFiles").unwrap_or_default();
            let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_default();

            for base in &[local_app_data, program_files, program_files_x86] {
                if !base.is_empty() && Path::new(&format!("{}/{}", base, app_path)).exists() {
                    return true;
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(bin) = editor.linux_bin {
            if Command::new("which")
                .arg(bin)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return true;
            }
        }
    }

    false
}

pub fn open_path_in_editor(editor_id: &str, path: &str) -> Result<(), String> {
    let editor = SUPPORTED_EDITORS
        .iter()
        .find(|e| e.id == editor_id)
        .ok_or_else(|| format!("Editor '{}' not supported", editor_id))?;

    #[cfg(target_os = "macos")]
    {
        // Terminal editors: open Terminal.app at the path
        if editor.is_terminal_editor {
            // Use osascript to open Terminal and run the editor
            let script = format!(
                "tell application \"Terminal\" to do script \"{} {}\"",
                editor.binary_name,
                path.replace('"', "\\\"")
            );
            let status = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .status()
                .map_err(|e| e.to_string())?;
            if status.success() {
                return Ok(());
            }
            // Fallback: just open Terminal at the directory
            let _ = Command::new("open")
                .arg("-a")
                .arg("Terminal")
                .arg(path)
                .status();
            return Ok(());
        }

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

        // Try exact app name in /Applications
        if let Some(app_name) = editor.macos_app_name {
            for base in &[
                "/Applications".to_string(),
                std::env::var("HOME")
                    .map(|h| format!("{}/Applications", h))
                    .unwrap_or_default(),
            ] {
                let app_path = format!("{}/{}", base, app_name);
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
            }
        }

        // Scan /Applications for version-specific app names
        if let Some(scan_name) = editor.macos_scan_name {
            for base in &[
                "/Applications".to_string(),
                std::env::var("HOME")
                    .map(|h| format!("{}/Applications", h))
                    .unwrap_or_default(),
            ] {
                if let Ok(entries) = std::fs::read_dir(base) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.contains(scan_name) && name.ends_with(".app") {
                            let app_path = format!("{}/{}", base, name);
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
    }

    #[cfg(target_os = "windows")]
    {
        // Try running the binary directly if it's in PATH
        if Command::new("cmd")
            .arg("/c")
            .arg(editor.binary_name)
            .arg(path)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Ok(());
        }

        // Try known Windows paths
        if let Some(app_path) = editor.windows_app_path {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let program_files = std::env::var("ProgramFiles").unwrap_or_default();
            let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_default();

            for base in &[local_app_data, program_files, program_files_x86] {
                if base.is_empty() {
                    continue;
                }
                let full_path = format!("{}/{}", base, app_path);
                if Path::new(&full_path).exists() {
                    let status = Command::new(&full_path)
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
