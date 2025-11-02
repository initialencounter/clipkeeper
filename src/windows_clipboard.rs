use std::fs;

use crate::get_clipkeeper_data_path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
            GetClipboardFormatNameW, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
        },
        Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
    },
};

#[cfg(windows)]
use windows::Win32::Foundation::HGLOBAL;

/// Windows 剪贴板格式数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClipboardFormat {
    /// 格式 ID
    pub format_id: u32,
    /// 格式名称（如果是自定义格式）
    pub format_name: Option<String>,
    /// 格式数据（Base64 编码）
    #[serde(with = "base64_serde")]
    pub data: Vec<u8>,
}

/// Windows 剪贴板完整快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClipboardSnapshot {
    pub formats: Vec<WindowsClipboardFormat>,
}

impl WindowsClipboardSnapshot {
    pub fn save_to_file(&self, path: Option<std::path::PathBuf>) -> Result<std::path::PathBuf> {
        let file_path = match path {
            Some(p) => p,
            None => {
                let file_dir = get_clipkeeper_data_path();
                if !fs::exists(&file_dir).unwrap() {
                    fs::create_dir_all(&file_dir)?;
                }
                file_dir.join("clipboard_snapshot.json")
            }
        };

        let json_content = serde_json::to_string_pretty(self)?;
        std::fs::write(&file_path, json_content)?;

        Ok(file_path)
    }

    pub fn load_from_file(path: Option<std::path::PathBuf>) -> Result<WindowsClipboardSnapshot> {
        let file_path = match path {
            Some(p) => p,
            None => get_clipkeeper_data_path().join("clipboard_snapshot.json"),
        };

        let json_content = std::fs::read_to_string(&file_path)?;
        let snapshot: WindowsClipboardSnapshot = serde_json::from_str(&json_content)?;

        Ok(snapshot)
    }
}

mod base64_serde {
    use base64::{Engine as _, engine::general_purpose};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let base64_str = general_purpose::STANDARD.encode(data);
        base64_str.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base64_str: String = String::deserialize(deserializer)?;
        general_purpose::STANDARD
            .decode(&base64_str)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(windows)]
/// 获取剪贴板格式的名称
unsafe fn get_format_name(format_id: u32) -> Option<String> {
    // 标准格式不需要获取名称
    if format_id < 0xC000 {
        return None;
    }

    let mut buffer = vec![0u16; 256];
    let len = unsafe { GetClipboardFormatNameW(format_id, &mut buffer) };

    if len > 0 {
        let name = String::from_utf16_lossy(&buffer[..len as usize]);
        Some(name)
    } else {
        None
    }
}

#[cfg(windows)]
/// 注册或获取剪贴板格式 ID
unsafe fn register_format(name: &str) -> u32 {
    let wide_chars: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { RegisterClipboardFormatW(windows::core::PCWSTR(wide_chars.as_ptr())) }
}

#[cfg(windows)]
/// 从剪贴板数据句柄读取数据
/// 注意: 剪贴板返回的句柄由剪贴板管理，不应该手动 Unlock
unsafe fn read_clipboard_data(handle: HGLOBAL) -> Result<Vec<u8>> {
    unsafe {
        let data_size = GlobalSize(handle);
        if data_size == 0 {
            return Ok(Vec::new());
        }

        let data_ptr = GlobalLock(handle);
        if data_ptr.is_null() {
            anyhow::bail!("无法锁定全局内存");
        }

        // 复制数据到新的 Vec
        let data = std::slice::from_raw_parts(data_ptr as *const u8, data_size).to_vec();
        
        // 解锁内存（但不释放，因为它属于剪贴板）
        let _ = GlobalUnlock(handle);

        Ok(data)
    }
}

#[cfg(windows)]
/// 将数据写入全局内存句柄（用于设置到剪贴板）
unsafe fn write_global_data(data: &[u8]) -> Result<HGLOBAL> {
    unsafe {
        let mem_handle = GlobalAlloc(GMEM_MOVEABLE, data.len())?;
        if mem_handle.0.is_null() {
            anyhow::bail!("无法分配全局内存");
        }

        let mem_ptr = GlobalLock(mem_handle);
        if mem_ptr.is_null() {
            // Note: GlobalFree is not needed as ownership is transferred to clipboard
            anyhow::bail!("无法锁定全局内存");
        }

        std::ptr::copy_nonoverlapping(data.as_ptr(), mem_ptr as *mut u8, data.len());
        let _ = GlobalUnlock(mem_handle);

        Ok(mem_handle)
    }
}

#[cfg(windows)]
/// 获取当前剪贴板的所有格式和数据（完整快照）
pub fn get_windows_clipboard_snapshot() -> Result<WindowsClipboardSnapshot> {
    unsafe {
        OpenClipboard(HWND::default())?;

        let mut formats = Vec::new();
        let mut format_id = 0u32;

        loop {
            format_id = EnumClipboardFormats(format_id);
            if format_id == 0 {
                break;
            }

            match GetClipboardData(format_id) {
                Ok(data_handle) => {
                    if data_handle.0.is_null() {
                        eprintln!("警告: 格式 {} 的数据句柄为空", format_id);
                        continue;
                    }
                    
                    let global_handle = HGLOBAL(data_handle.0);
                    match read_clipboard_data(global_handle) {
                        Ok(data) => {
                            let format_name = get_format_name(format_id);

                            formats.push(WindowsClipboardFormat {
                                format_id,
                                format_name,
                                data,
                            });
                        }
                        Err(e) => {
                            eprintln!("警告: 读取格式 {} 数据失败: {}", format_id, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("警告: 无法获取格式 {} 的数据: {:?}", format_id, e);
                }
            }
        }

        let _ = CloseClipboard();

        Ok(WindowsClipboardSnapshot { formats })
    }
}

#[cfg(windows)]
/// 恢复剪贴板到指定快照（完整恢复所有格式）
pub fn restore_windows_clipboard_snapshot(snapshot: &WindowsClipboardSnapshot) -> Result<()> {
    unsafe {
        OpenClipboard(HWND::default())?;

        if let Err(e) = EmptyClipboard() {
            let _ = CloseClipboard();
            anyhow::bail!("无法清空剪贴板: {}", e);
        }

        for format in &snapshot.formats {
            let format_id = if let Some(ref name) = format.format_name {
                register_format(name)
            } else {
                format.format_id
            };

            match write_global_data(&format.data) {
                Ok(global_handle) => {
                    let data_handle = windows::Win32::Foundation::HANDLE(global_handle.0);
                    if let Err(e) = SetClipboardData(format_id, data_handle) {
                        eprintln!("警告: 设置格式 {} 失败: {}", format_id, e);
                    }
                }
                Err(e) => {
                    eprintln!("警告: 分配格式 {} 的内存失败: {}", format_id, e);
                }
            }
        }

        let _ = CloseClipboard();
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn get_windows_clipboard_snapshot() -> Result<WindowsClipboardSnapshot> {
    anyhow::bail!("Windows 剪贴板 API 仅在 Windows 平台可用");
}

#[cfg(not(windows))]
pub fn restore_windows_clipboard_snapshot(_snapshot: &WindowsClipboardSnapshot) -> Result<()> {
    anyhow::bail!("Windows 剪贴板 API 仅在 Windows 平台可用");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_api() {
        // 步骤1: 获取当前剪贴板内容（所有格式）
        println!("1. 获取当前剪贴板内容（使用 Windows API - 捕获所有格式）");
        let snapshot = get_windows_clipboard_snapshot().unwrap();
        println!("   捕获了 {} 个格式:", snapshot.formats.len());
        for (i, format) in snapshot.formats.iter().enumerate() {
            println!(
                "   [{:2}] 格式 ID: {:5}, 名称: {:30}, 数据: {} 字节",
                i + 1,
                format.format_id,
                format.format_name.as_deref().unwrap_or("<标准格式>"),
                format.data.len()
            );
        }

        // 步骤2: 保存到文件
        println!("\n2. 保存 Windows 快照到文件");
        let file_path = snapshot.save_to_file(None).unwrap();
        println!("   已保存到: {:?}", file_path);
        println!(
            "   文件大小: {} 字节",
            std::fs::metadata(&file_path).unwrap().len()
        );

        // 步骤4: 从文件加载快照
        println!("\n4. 从文件加载 Windows 快照");
        let loaded_snapshot = WindowsClipboardSnapshot::load_from_file(None).unwrap();
        println!("   加载成功");
        println!("   格式数量: {}", loaded_snapshot.formats.len());

        // 步骤5: 恢复剪贴板（完整恢复所有格式）
        println!("\n5. 恢复剪贴板到快照状态（完整恢复所有格式）");
        restore_windows_clipboard_snapshot(&loaded_snapshot).unwrap();
        println!("   恢复成功");
    }
}
