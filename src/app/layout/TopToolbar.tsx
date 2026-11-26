import { ActiveDocumentToolbar } from "./ActiveDocumentToolbar";

export function TopToolbar() {
  return (
    <header className="toolbar">
      <section className="toolbar-section toolbar-section-center" aria-label="活动文档工具区">
        <ActiveDocumentToolbar />
      </section>
    </header>
  );
}
