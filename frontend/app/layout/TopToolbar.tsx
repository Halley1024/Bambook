import { ActiveDocumentToolbar } from "./ActiveDocumentToolbar";
import { AppMenuActions } from "./AppMenuActions";

export function TopToolbar() {
  return (
    <header className="toolbar">
      <section className="toolbar-section toolbar-section-center" aria-label="活动文档工具区">
        <ActiveDocumentToolbar />
      </section>

      <section className="toolbar-section toolbar-section-right" aria-label="文件与设置区">
        <AppMenuActions />
      </section>
    </header>
  );
}
