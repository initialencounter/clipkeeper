# Clipboard Persistence

一个用于保存和恢复 Windows 剪贴板内容的 Rust 库。

## 功能特性

- 📋 捕获剪贴板所有格式的完整快照
- 💾 将剪贴板内容保存到 JSON 文件
- 🔄 从文件恢复剪贴板内容
- 🎯 支持标准格式和自定义格式
- ✅ Windows 平台原生 API 支持

## 安装

将以下内容添加到 `Cargo.toml`：

```toml
[dependencies]
clipkeeper = "0.1.0"
```

## 快速开始

```rust
use clipkeeper::{
    get_windows_clipboard_snapshot,
    restore_windows_clipboard_snapshot,
    WindowsClipboardSnapshot,
};

fn main() -> anyhow::Result<()> {
    // 1. 获取当前剪贴板快照
    let snapshot = get_windows_clipboard_snapshot()?;
    println!("捕获了 {} 个格式", snapshot.formats.len());

    // 2. 保存到文件（默认路径：~AppData/Roaming/clipkeeper/clipboard_snapshot.json）
    let file_path = snapshot.save_to_file(None)?;
    println!("已保存到: {:?}", file_path);

    // 3. 从文件加载快照
    let loaded_snapshot = WindowsClipboardSnapshot::load_from_file(None)?;

    // 4. 恢复剪贴板
    restore_windows_clipboard_snapshot(&loaded_snapshot)?;
    println!("剪贴板已恢复");

    Ok(())
}
```

## 自定义保存路径

```rust
use std::path::PathBuf;

// 保存到自定义路径
let custom_path = PathBuf::from("my_clipboard.json");
snapshot.save_to_file(Some(custom_path.clone()))?;

// 从自定义路径加载
let snapshot = WindowsClipboardSnapshot::load_from_file(Some(custom_path))?;
```

## 运行示例

```bash
cargo run --example basic
```

## 平台支持

- ✅ Windows (x86_64, i686)
- ❌ macOS（待支持）
- ❌ Linux（待支持）

## 许可证

AGPL-3.0

## 作者

[inintencunter](https://github.com/initialencounter)
