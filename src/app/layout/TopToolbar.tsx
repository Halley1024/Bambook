import { ActiveDocumentToolbar } from "./ActiveDocumentToolbar";
import { AppMenuActions } from "./AppMenuActions";
import { ApplicationIdentity } from "./ApplicationIdentity";

export function TopToolbar() {
  return (
    <header className="toolbar">
      <section className="toolbar-section toolbar-section-left" aria-label="当前文档类型">
        <ApplicationIdentity />
      </section>

      <section className="toolbar-section toolbar-section-center" aria-label="活动文档工具区">
        <ActiveDocumentToolbar />
      </section>

      <section className="toolbar-section toolbar-section-right" aria-label="文件与设置区">
        <AppMenuActions />
      </section>
    </header>
  );
}
