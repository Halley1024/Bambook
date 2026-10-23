/// The invoke names and request/response DTOs are kept compatible with the frontend.
pub(super) fn handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // 应用程序工作流workflow
        crate::app::commands::open_pdf_document,
        crate::app::commands::close_pdf_document,
        crate::app::commands::authenticate_pdf_document,
        crate::app::commands::persist_pdf_document,
        crate::app::commands::save_markdown_document,
        crate::app::commands::open_markdown_document,
        crate::app::commands::save_markdown_document_as,
        crate::app::commands::close_markdown_document,
        crate::app::commands::load_document_history,
        crate::app::commands::reconcile_document_storage,
        crate::app::commands::export_document_package,
        // 平台相关的命令
        crate::platform::desktop::commands::open_app_data_directory,
        crate::platform::desktop::commands::reveal_in_file_manager,
        crate::platform::desktop::commands::open_directory_in_file_manager,
        crate::platform::desktop::commands::open_external_url,
        // 通用文件类型命令
        crate::features::documents::commands::detect_document_kind, // 检查文件类型
        crate::features::documents::commands::list_markdown_directory,
        crate::features::documents::commands::list_supported_directory,
        crate::features::documents::commands::rename_markdown_document,
        crate::features::documents::commands::rename_pdf_document,
        // pdf文件相关处理命令
        crate::features::pdf::commands::get_pdf_page_structure,
        crate::features::pdf::commands::render_pdf_page,
        crate::features::pdf::storage_commands::load_pdf_bookmarks,
        crate::features::pdf::storage_commands::save_pdf_bookmarks,
        crate::features::pdf::storage_commands::load_pdf_reading_state,
        crate::features::pdf::storage_commands::save_pdf_reading_state,
        // markdown文件相关处理命令
        crate::features::markdown::commands::create_markdown_document,
        crate::features::markdown::commands::create_untitled_markdown_document,
        crate::features::markdown::commands::get_markdown_structure,
        crate::features::markdown::commands::apply_markdown_edits,
        crate::features::markdown::commands::export_markdown_html,
        crate::features::markdown::commands::export_markdown_pdf,
        crate::features::markdown::commands::save_untitled_markdown_recovery,
        crate::features::markdown::commands::load_untitled_markdown_recovery,
        crate::features::markdown::commands::clear_untitled_markdown_recovery,
        // 搜索模块功能命令
        crate::features::search::commands::search_document,
        crate::features::search::commands::cancel_document_search,
        // 软件设置中心模块
        crate::features::settings::commands::load_settings,
        crate::features::settings::commands::save_settings,
        // AI小助手模块：翻译、LLM、MCP
        crate::features::assistant::translation::commands::translate_text,
        crate::features::assistant::translation::commands::test_translation_connection,
        crate::features::assistant::llm::commands::ask_llm,
        crate::features::assistant::llm::commands::test_llm_connection,
        crate::features::assistant::mcp::commands::list_mcp_tools,
        // 文件加载和关联处理
        crate::features::library::commands::load_document_library,
        crate::features::library::commands::inspect_document_relink,
        crate::features::library::commands::relink_document,
        // 历史记录模块
        crate::features::history::commands::record_document_closed,
        crate::features::history::commands::record_document_accessed,
        crate::features::history::commands::confirm_reopened_closed_document,
        crate::features::history::commands::clear_document_history,
        crate::features::history::commands::import_legacy_document_history,
        // 工作区模块
        crate::features::workspace::commands::load_workspace_state,
        crate::features::workspace::commands::save_workspace_state,
        // 注释模块
        crate::features::annotations::commands::load_annotations,
        crate::features::annotations::commands::save_annotations,
        crate::features::annotations::commands::migrate_pdf_annotations,
        crate::features::annotations::export_commands::export_annotations_markdown
    ]
}
