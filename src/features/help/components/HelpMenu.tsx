import { CircleHelp, HeartHandshake, Info, MessageSquareText, ScrollText } from "lucide-react";
import { DropdownMenu, type DropdownMenuGroup } from "../../../components/menus/DropdownMenu";

const unavailable = "帮助页面将在后续阶段接入";

export function HelpMenu() {
  const groups: DropdownMenuGroup[] = [{
    id: "help",
    items: [
      { id: "credits", label: "鸣谢", icon: HeartHandshake, disabled: true, title: unavailable },
      { id: "changelog", label: "更新日志", icon: ScrollText, disabled: true, title: unavailable },
      { id: "feedback", label: "留言反馈", icon: MessageSquareText, disabled: true, title: unavailable },
      { id: "about", label: "关于", icon: Info, disabled: true, title: unavailable },
    ],
  }];
  return <DropdownMenu label="帮助" icon={CircleHelp} groups={groups} />;
}
