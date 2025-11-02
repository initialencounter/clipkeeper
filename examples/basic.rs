use clipkeeper::{
    get_windows_clipboard_snapshot, restore_windows_clipboard_snapshot,
    WindowsClipboardSnapshot,
};

fn main() -> anyhow::Result<()> {
    println!("=== 剪贴板持久化示例 ===\n");

    // 1. 获取当前剪贴板快照
    println!("📋 正在获取剪贴板内容...");
    let snapshot = get_windows_clipboard_snapshot()?;
    println!("✅ 成功捕获 {} 个格式\n", snapshot.formats.len());

    // 显示捕获的格式信息
    for (i, format) in snapshot.formats.iter().enumerate() {
        println!(
            "  [{}] ID: {}, 名称: {}, 数据大小: {} 字节",
            i + 1,
            format.format_id,
            format.format_name.as_deref().unwrap_or("<标准格式>"),
            format.data.len()
        );
    }

    // 2. 保存到文件
    println!("\n💾 正在保存到文件...");
    let file_path = snapshot.save_to_file(None)?;
    println!("✅ 已保存到: {:?}", file_path);

    // 3. 提示用户更改剪贴板
    println!("\n⏸️  请复制一些新内容到剪贴板，然后按 Enter 继续...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // 4. 从文件加载并恢复剪贴板
    println!("🔄 正在从文件恢复剪贴板...");
    let loaded_snapshot = WindowsClipboardSnapshot::load_from_file(None)?;
    restore_windows_clipboard_snapshot(&loaded_snapshot)?;
    println!("✅ 剪贴板已恢复到之前的状态");

    println!("\n✨ 完成！你可以粘贴（Ctrl+V）验证内容是否已恢复。");

    Ok(())
}
