import { useEffect, useState } from "react";
import { Bell, Check, Database, FolderOpen, Palette, RotateCcw, Rocket, X } from "lucide-react";
import { useSettings } from "../hooks/useSettings";
import { selectDirectoryPath } from "../../../platform/fileDialog";
import type { DocumentImportMode, StartupBehavior, ThemePreference } from "../types";

export function SettingsPanel() {
  const workspace = useSettings();
  const { settings } = workspace;
  const [section, setSection] = useState<"general" | "appearance" | "notifications" | "data">("general");
  const [confirmReset, setConfirmReset] = useState(false);

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => event.key === "Escape" && workspace.closePanel();
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [workspace.closePanel]);

  return (
    <div className="settings-dialog-backdrop" onMouseDown={workspace.closePanel}>
      <section className="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title"
        onMouseDown={(event) => event.stopPropagation()}>
        <header className="settings-dialog-header">
          <div><h2 id="settings-title">设置中心</h2><p>调整 Bambook 的启动方式和界面外观。</p></div>
          <button type="button" onClick={workspace.closePanel} data-tooltip="关闭：退出设置中心" aria-label="关闭设置"><X size={19} /></button>
        </header>
        <div className="settings-dialog-body">
          <nav className="settings-navigation" aria-label="设置分类">
            <SettingsNavButton active={section === "general"} icon={Rocket} label="通用" onClick={() => setSection("general")} />
            <SettingsNavButton active={section === "appearance"} icon={Palette} label="外观" onClick={() => setSection("appearance")} />
            <SettingsNavButton active={section === "notifications"} icon={Bell} label="通知提醒" onClick={() => setSection("notifications")} />
            <SettingsNavButton active={section === "data"} icon={Database} label="数据与维护" onClick={() => setSection("data")} />
          </nav>
          <div className="settings-content">
            {section === "general" && <GeneralSettings />}
            {section === "appearance" && <AppearanceSettings />}
            {section === "notifications" && <NotificationSettings />}
            {section === "data" && (
              <section className="settings-section">
                <h3>数据与维护</h3><p className="settings-section-description">查看应用数据或恢复默认偏好。</p>
                <div className="settings-row">
                  <div><strong>应用数据文件夹</strong><small>%LOCALAPPDATA%\Bambook</small></div>
                  <button className="settings-action-button" data-tooltip="打开文件夹：在系统文件管理器中查看应用数据"
                    onClick={() => void workspace.openDataDirectory()}><FolderOpen size={16} />打开文件夹</button>
                </div>
                <div className="settings-row danger-row">
                  <div><strong>恢复默认设置</strong><small>不会删除用户的 PDF、Markdown 或批注文件。</small></div>
                  {!confirmReset ? (
                    <button className="settings-action-button danger" data-tooltip="恢复默认：重置界面和启动偏好"
                      onClick={() => setConfirmReset(true)}><RotateCcw size={16} />恢复默认</button>
                  ) : (
                    <div className="settings-confirm-actions">
                      <button data-tooltip="取消：保留当前设置" onClick={() => setConfirmReset(false)}>取消</button>
                      <button className="danger" data-tooltip="确认恢复：立即重置所有设置"
                        onClick={() => { void workspace.resetSettings(); setConfirmReset(false); }}>确认恢复</button>
                    </div>
                  )}
                </div>
              </section>
            )}
            {section === "data" && <StorageSettings />}
          </div>
        </div>
      </section>
    </div>
  );
}

function NotificationSettings() {
  const { settings, updateSettings } = useSettings();
  return <section className="settings-section"><h3>通知提醒</h3>
    <p className="settings-section-description">控制清除书签等不可逆批量操作前是否需要再次确认。</p>
    <div className="settings-row"><div><strong>操作确认提醒</strong><small>清除当前页或全部书签前显示确认弹窗。</small></div>
      <button type="button" role="switch" aria-checked={settings.confirmationRemindersEnabled}
        className={`settings-switch ${settings.confirmationRemindersEnabled ? "active" : ""}`}
        onClick={() => void updateSettings({ confirmationRemindersEnabled: !settings.confirmationRemindersEnabled })}><span /></button>
    </div>
  </section>;
}

function StorageSettings() {
  const { settings, updateSettings } = useSettings();
  const chooseDirectory = async (kind: "pdf" | "markdown") => {
    const path = await selectDirectoryPath();
    if (!path) return;
    await updateSettings(kind === "pdf" ? { pdfStorageDir: path } : { markdownStorageDir: path });
  };
  const modes: Array<{ value: DocumentImportMode; label: string }> = [
    { value: "copy", label: "添加文件副本" },
    { value: "link", label: "仅添加链接" },
  ];
  return <section className="settings-section settings-storage-section">
    <h3>文档存储</h3>
    <p className="settings-section-description">分别设置 PDF 与 Markdown 的托管目录，以及首次保存时的入库方式。</p>
    {(["pdf", "markdown"] as const).map((kind) => {
      const isPdf = kind === "pdf";
      const directory = isPdf ? settings.pdfStorageDir : settings.markdownStorageDir;
      const mode = isPdf ? settings.pdfImportMode : settings.markdownImportMode;
      return <div className="settings-row stacked settings-storage-row" key={kind}>
        <div><strong>{isPdf ? "PDF 文档" : "Markdown 文档"}</strong><small>{directory || "默认：%LOCALAPPDATA%\\Bambook\\storage"}</small></div>
        <div className="settings-storage-actions">
          <button className="settings-action-button" onClick={() => void chooseDirectory(kind)}><FolderOpen size={16} />选择保存位置</button>
          {directory && <button className="settings-action-button" onClick={() => void updateSettings(isPdf ? { pdfStorageDir: undefined } : { markdownStorageDir: undefined })}>恢复默认</button>}
          <div className="settings-segmented">{modes.map((item) => <button key={item.value} className={mode === item.value ? "active" : ""}
            onClick={() => void updateSettings(isPdf ? { pdfImportMode: item.value } : { markdownImportMode: item.value })}>{item.label}</button>)}</div>
        </div>
      </div>;
    })}
    <p className="settings-safety-note">打开文件不会立即建库；首次保存时，“添加文件副本”会复制文件，“仅添加链接”只记录原文件的绝对路径。</p>
  </section>;
}

function GeneralSettings() {
  const { settings, updateSettings } = useSettings();
  const choices: Array<{ value: StartupBehavior; title: string; description: string }> = [
    { value: "restoreWorkspace", title: "恢复上次工作区", description: "重新打开上次保留的文档标签和活动文档。" },
    { value: "newMarkdown", title: "新建空白 Markdown", description: "每次启动时创建一个未命名 Markdown 文档。" },
    { value: "startPage", title: "显示开始页面", description: "启动后不打开文档，显示打开文件和新建 Markdown。" },
  ];
  return <section className="settings-section"><h3>通用</h3><p className="settings-section-description">选择 Bambook 启动后显示的内容。</p>
    <fieldset className="settings-choice-list"><legend>启动时</legend>{choices.map((choice) => (
      <button key={choice.value} className={`settings-choice ${settings.startupBehavior === choice.value ? "active" : ""}`}
        data-tooltip={`${choice.title}：${choice.description}`}
        onClick={() => void updateSettings({ startupBehavior: choice.value })}>
        <span className="settings-choice-check">{settings.startupBehavior === choice.value && <Check size={15} />}</span>
        <span><strong>{choice.title}</strong><small>{choice.description}</small></span>
      </button>
    ))}</fieldset>
    <p className="settings-safety-note">检测到崩溃恢复内容时，软件仍会优先询问是否恢复，避免内容丢失。</p>
  </section>;
}

function AppearanceSettings() {
  const { settings, setTheme, toggleAnimations } = useSettings();
  const themes: Array<{ value: ThemePreference; label: string }> = [
    { value: "system", label: "跟随系统" }, { value: "light", label: "浅色" }, { value: "dark", label: "深色" },
  ];
  return <section className="settings-section"><h3>外观</h3><p className="settings-section-description">保持 Bambook 简洁、舒适的阅读界面。</p>
    <div className="settings-row stacked"><div><strong>主题</strong><small>改变应用外壳、菜单和文档工作区的配色。</small></div>
      <div className="settings-segmented">{themes.map((theme) => <button key={theme.value}
        className={settings.theme === theme.value ? "active" : ""} data-tooltip={`主题：切换为${theme.label}`}
        onClick={() => void setTheme(theme.value)}>{theme.label}</button>)}</div>
    </div>
    <div className="settings-row"><div><strong>界面动画</strong><small>控制面板切换、菜单和弹窗的过渡效果。</small></div>
      <button type="button" role="switch" aria-checked={settings.animationsEnabled}
        data-tooltip={`${settings.animationsEnabled ? "关闭" : "开启"}动画：控制界面过渡效果`}
        className={`settings-switch ${settings.animationsEnabled ? "active" : ""}`} onClick={() => void toggleAnimations()}><span /></button>
    </div>
  </section>;
}

function SettingsNavButton({ active, icon: Icon, label, onClick }: { active: boolean; icon: typeof Rocket; label: string; onClick: () => void }) {
  return <button className={active ? "active" : ""} data-tooltip={`设置分类：打开${label}设置`} onClick={onClick}>
    <Icon size={17} /><span>{label}</span>
  </button>;
}
