import { BookOpen, Clock3, Library, Network, ShieldCheck, Star } from "lucide-react";
import { DocumentFileNavigator } from "./DocumentFileNavigator";

export type LibrarySection = "all" | "recent" | "favorites";

export function WorkspaceNavigation({ section, onNavigate }: {
  section: LibrarySection | null;
  onNavigate: (section: LibrarySection) => void;
}) {
  return <aside className="workspace-navigation" aria-label="工作区导航">
    <header className="workspace-navigation-brand"><span><BookOpen size={21} /></span><div><strong>我的工作区</strong><small>一方安静的思考空间</small></div></header>
    <nav className="workspace-library-nav" aria-label="文档集合">
      {([{ id: "all", label: "文档库", Icon: Library }, { id: "recent", label: "最近阅读", Icon: Clock3 }, { id: "favorites", label: "我的收藏", Icon: Star }] as const).map(({ id, label, Icon }) =>
        <button key={id} type="button" className={section === id ? "active" : ""} aria-current={section === id ? "page" : undefined} onClick={() => onNavigate(id)}><Icon size={19} /><span>{label}</span></button>)}
      <button type="button" disabled data-tooltip="知识图谱（暂未开发）"><Network size={19} /><span>知识图谱</span><small>待开发</small></button>
    </nav>
    <div className="workspace-folder-tree"><DocumentFileNavigator directoryOnly /></div>
    <footer className="workspace-navigation-footer"><ShieldCheck size={15} /><span>让阅读与写作自然相连</span></footer>
  </aside>;
}
