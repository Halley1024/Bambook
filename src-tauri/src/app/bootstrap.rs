use tauri::Manager;

use crate::app::state::AppState;
use crate::shared::error::AppError;

/// 启动应用程序
///
///  tauri::Builder::default() 创建 Tauri 应用的默认构造器，里面是一个对接操作系统Web视图的运行时
/// .plugin(tauri_plugin_dialog::init()) 添加对话框插件，让应用能够弹出操作系统原生的文件选择、保存以及消息提示框
/// .setup(|app: &mut tauri::App| { ... }) 设置应用程序的初始化逻辑，这里使用闭包来实现，闭包接收一个 App 引用作为参数，并返回一个 Result<(), AppError> 类型的值
pub(crate) fn run() {
    tauri::Builder::default() // 创建 Tauri 应用的默认构造器，里面是一个对接操作系统Web视图的运行时
        .plugin(tauri_plugin_dialog::init()) // 添加对话框插件，让应用能够弹出操作系统原生的文件选择、保存以及消息提示框
        .setup(|app: &mut tauri::App| {
            // 提供一个钩子函数，用闭包函数对app进行初始化
            let data_dir: std::path::PathBuf = app
                .path() // 获取一个路径解析器的引用
                .local_data_dir() // 获取应用程序的本地数据目录
                .map_err(|_| AppError::MissingAppDataDir)?
                .join("Bambook");
            AppState::new(data_dir)?.register(app); // 注册应用程序的全局状态信息
            Ok(())
        })
        .invoke_handler(super::ipc::handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
