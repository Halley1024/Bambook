/// The invoke names and request/response DTOs are kept compatible with the frontend.
pub(super) fn handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // 应用程序工作流workflow
        crate::app::commands::open_pdf_document, // 打开PDF文件
        crate::app::commands::close_pdf_document, // 关闭PDF文件
        crate::app::commands::authenticate_pdf_document, // 验证PDF文件
        crate::app::commands::persist_pdf_document, // 持久化PDF文件
        crate::app::commands::save_markdown_document, // 保存Markdown文件
        crate::app::commands::open_markdown_document, // 打开Markdown文件
        crate::app::commands::save_markdown_document_as, // 另存为Markdown文件
        crate::app::commands::close_markdown_document, // 关闭Markdown文件
        crate::app::commands::load_document_history, // 加载文档历史记录
        crate::app::commands::reconcile_document_storage, // 同步文档存储
        crate::app::commands::export_document_package, // 导出文档包
        // 平台相关的命令
        crate::platform::desktop::commands::open_app_data_directory, // 打开应用程序的数据目录
        crate::platform::desktop::commands::reveal_in_file_manager,  // 在文件管理器中显示文件
        crate::platform::desktop::commands::open_directory_in_file_manager, // 打开文件管理器中的目录
        crate::platform::desktop::commands::open_external_url,              // 打开外部URL
        // 通用文件类型命令
        crate::features::documents::commands::detect_document_kind, // 检查文件类型
        crate::features::documents::commands::list_markdown_directory, //  列出Markdown文件目录
        crate::features::documents::commands::list_supported_directory, // 列出支持的文件目录
        crate::features::documents::commands::rename_markdown_document, // 重命名Markdown文件
        crate::features::documents::commands::rename_pdf_document,  // 重命名PDF文件
        // pdf文件相关处理命令
        crate::features::pdf::commands::get_pdf_page_structure, // 获取PDF页面结构
        crate::features::pdf::commands::render_pdf_page,        // 渲染PDF页面
        crate::features::pdf::storage_commands::load_pdf_bookmarks, // 加载PDF书签
        crate::features::pdf::storage_commands::save_pdf_bookmarks, // 保存PDF书签
        crate::features::pdf::storage_commands::load_pdf_reading_state, // 加载PDF阅读状态
        crate::features::pdf::storage_commands::save_pdf_reading_state, // 保存PDF阅读状态
        // markdown文件相关处理命令
        crate::features::markdown::commands::create_markdown_document, // 创建Markdown文件
        crate::features::markdown::commands::create_untitled_markdown_document, // 创建未命名的Markdown文件
        crate::features::markdown::commands::get_markdown_structure,            // 获取Markdown结构
        crate::features::markdown::commands::apply_markdown_edits,              // 应用Markdown编辑
        crate::features::markdown::commands::export_markdown_html, // 导出Markdown为HTML
        crate::features::markdown::commands::export_markdown_pdf,  // 导出Markdown为PDF
        crate::features::markdown::commands::save_untitled_markdown_recovery, // 保存未命名的Markdown文件
        crate::features::markdown::commands::load_untitled_markdown_recovery, // 加载未命名的Markdown文件
        crate::features::markdown::commands::clear_untitled_markdown_recovery, // 清除未命名的Markdown文件
        // 搜索模块功能命令
        crate::features::search::commands::search_document, // 搜索文档
        crate::features::search::commands::cancel_document_search, // 取消文档搜索
        // 软件设置中心模块
        crate::features::settings::commands::load_settings, // 加载设置
        crate::features::settings::commands::save_settings, // 保存设置
        // AI小助手模块：翻译、LLM、MCP TODO:Agent
        crate::features::assistant::translation::commands::translate_text, // 翻译文本
        crate::features::assistant::translation::commands::test_translation_connection, // 测试翻译连接
        crate::features::assistant::llm::commands::ask_llm,                             // 询问LLM
        crate::features::assistant::llm::commands::test_llm_connection, // 测试LLM连接
        crate::features::assistant::mcp::commands::list_mcp_tools,      // 列出MCP工具
        // 文件加载和关联处理
        crate::features::library::commands::load_document_library, // 加载文档库
        crate::features::library::commands::inspect_document_relink, // 检查文档重链接
        crate::features::library::commands::relink_document,       // 重链接文档
        // 历史记录模块
        crate::features::history::commands::record_document_closed, // 记录文档关闭
        crate::features::history::commands::record_document_accessed, // 记录文档访问
        crate::features::history::commands::confirm_reopened_closed_document, // 确认重新打开的关闭文档
        crate::features::history::commands::clear_document_history,           // 清除文档历史记录
        crate::features::history::commands::import_legacy_document_history, // 导入旧版文档历史记录
        // 工作区模块
        crate::features::workspace::commands::load_workspace_state, // 加载工作区状态
        crate::features::workspace::commands::save_workspace_state, // 保存工作区状态
        // 注释模块
        crate::features::annotations::commands::load_annotations, // 加载注释
        crate::features::annotations::commands::save_annotations, // 保存注释
        crate::features::annotations::commands::migrate_pdf_annotations, // 迁移PDF注释
        crate::features::annotations::export_commands::export_annotations_markdown // 导出注释为Markdown
    ]
}
