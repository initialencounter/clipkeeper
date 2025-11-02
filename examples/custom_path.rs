use clipkeeper::{
    get_windows_clipboard_snapshot, restore_windows_clipboard_snapshot,
};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("=== 自定义路径保存示例 ===\n");

    // 定义自定义保存路径
    let custom_path = PathBuf::from("my_clipboard_backup.json");

    // 1. 获取并保存到自定义路径
    println!("📋 获取剪贴板内容并保存到自定义路径...");
    let snapshot = get_windows_clipboard_snapshot()?;
    snapshot.save_to_file(Some(custom_path.clone()))?;
    println!("✅ 已保存到: {:?}\n", custom_path);

    // 2. 从自定义路径加载并恢复
    println!("🔄 从自定义路径加载...");
    let loaded_snapshot =
        clipkeeper::WindowsClipboardSnapshot::load_from_file(Some(custom_path.clone()))?;
    restore_windows_clipboard_snapshot(&loaded_snapshot)?;
    println!("✅ 剪贴板已恢复");

    // 3. 清理示例文件（可选）
    println!("\n🧹 清理示例文件...");
    std::fs::remove_file(&custom_path)?;
    println!("✅ 已删除: {:?}", custom_path);

    Ok(())
}
