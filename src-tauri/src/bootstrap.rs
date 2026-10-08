use tauri::Manager;

use crate::{commands, error::AppError, state::AppState};

/// 启动应用程序
///
///  tauri::Builder::default() 创建 Tauri 应用的默认构造器
/// .plugin(tauri_plugin_dialog::init()) 添加对话框插件，让应用能够弹出操作系统原生的文件选择、保存以及消息提示框
/// .setup(|app: &mut tauri::App| { ... }) 设置应用程序的初始化逻辑，这里使用闭包来实现，闭包接收一个 App 引用作为参数，并返回一个 Result<(), AppError> 类型的值
pub(crate) fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app: &mut tauri::App| {
            let data_dir: std::path::PathBuf = app
                .path()
                .local_data_dir() // 获取应用程序的本地数据目录
                .map_err(|_| AppError::MissingAppDataDir)?
                .join("Bambook");
            app.manage(AppState::new(data_dir)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::document_commands::detect_document_kind,
            commands::document_commands::create_markdown_document,
            commands::document_commands::reveal_in_file_manager,
            commands::document_commands::open_directory_in_file_manager,
            commands::document_commands::open_external_url,
            commands::document_commands::list_markdown_directory,
            commands::document_commands::list_supported_directory,
            commands::document_commands::rename_markdown_document,
            commands::document_commands::rename_pdf_document,
            commands::markdown_commands::open_markdown_document,
            commands::markdown_commands::create_untitled_markdown_document,
            commands::markdown_commands::get_markdown_structure,
            commands::markdown_commands::apply_markdown_edits,
            commands::markdown_commands::save_markdown_document,
            commands::markdown_commands::save_markdown_document_as,
            commands::markdown_commands::export_markdown_html,
            commands::markdown_commands::export_markdown_pdf,
            commands::markdown_commands::close_markdown_document,
            commands::markdown_commands::save_untitled_markdown_recovery,
            commands::markdown_commands::load_untitled_markdown_recovery,
            commands::markdown_commands::clear_untitled_markdown_recovery,
            commands::pdf_commands::open_pdf_document,
            commands::pdf_commands::authenticate_pdf_document,
            commands::pdf_commands::get_pdf_page_structure,
            commands::pdf_commands::render_pdf_page,
            commands::pdf_commands::close_pdf_document,
            commands::pdf_storage_commands::load_pdf_bookmarks,
            commands::pdf_storage_commands::persist_pdf_document,
            commands::pdf_storage_commands::save_pdf_bookmarks,
            commands::pdf_storage_commands::load_pdf_reading_state,
            commands::pdf_storage_commands::save_pdf_reading_state,
            commands::search_commands::search_document,
            commands::search_commands::cancel_document_search,
            commands::settings_commands::load_settings,
            commands::settings_commands::save_settings,
            commands::settings_commands::open_app_data_directory,
            commands::translation_commands::translate_text,
            commands::translation_commands::test_translation_connection,
            commands::llm_commands::ask_llm,
            commands::llm_commands::test_llm_connection,
            commands::mcp_commands::list_mcp_tools,
            commands::persistence_commands::load_document_history,
            commands::persistence_commands::load_document_library,
            commands::persistence_commands::inspect_document_relink,
            commands::persistence_commands::relink_document,
            commands::persistence_commands::reconcile_document_storage,
            commands::persistence_commands::export_document_package,
            commands::persistence_commands::record_document_closed,
            commands::persistence_commands::record_document_accessed,
            commands::persistence_commands::confirm_reopened_closed_document,
            commands::persistence_commands::clear_document_history,
            commands::persistence_commands::import_legacy_document_history,
            commands::persistence_commands::load_workspace_state,
            commands::persistence_commands::save_workspace_state,
            commands::annotation_commands::load_annotations,
            commands::annotation_commands::save_annotations,
            commands::annotation_commands::migrate_pdf_annotations,
            commands::export_commands::export_annotations_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
