#[cfg(windows)]
pub mod windows_clipboard;

#[cfg(windows)]
pub use windows_clipboard::{
    WindowsClipboardFormat, WindowsClipboardSnapshot, get_windows_clipboard_snapshot,
    restore_windows_clipboard_snapshot,
};


/// 从文件加载快照（默认路径为 ~AppData\\Roaming\\clipkeeper clipboard_snapshot.json）
pub fn get_clipkeeper_data_path() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("clipkeeper")
}