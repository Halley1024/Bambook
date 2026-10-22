import { Check, FolderOpen, Moon, Palette, Settings, SlidersHorizontal, Sparkles, Sun } from "lucide-react";
import { DropdownMenu, type DropdownMenuGroup } from "../../../components/menus/DropdownMenu";
import { useSettings } from "../hooks/useSettings";

export function SettingsMenu() {
  const settings = useSettings();
  const groups: DropdownMenuGroup[] = [
    {
      id: "quick-settings",
      items: [
        {
          id: "theme",
          label: "主题",
          icon: Palette,
          submenu: <ThemeMenu />,
        },
        {
          id: "animations",
          label: "界面动画",
          icon: Sparkles,
          shortcut: settings.settings.animationsEnabled ? "✓" : "",
          onSelect: () => void settings.toggleAnimations(),
        },
      ],
    },
    {
      id: "settings-actions",
      items: [
        { id: "data-folder", label: "打开数据文件夹", icon: FolderOpen, onSelect: () => void settings.openDataDirectory() },
        { id: "settings-center", label: "设置中心…", icon: SlidersHorizontal, onSelect: settings.openPanel },
      ],
    },
  ];
  return <DropdownMenu label="设置" icon={Settings} groups={groups} />;
}

function ThemeMenu() {
  const { settings, setTheme } = useSettings();
  const choices = [
    { value: "system" as const, label: "跟随系统", icon: Settings },
    { value: "light" as const, label: "浅色", icon: Sun },
    { value: "dark" as const, label: "深色", icon: Moon },
  ];
  return (
    <div className="settings-theme-menu" role="menu" aria-label="主题">
      {choices.map((choice) => {
        const Icon = choice.icon;
        const active = settings.theme === choice.value;
        return (
          <button key={choice.value} type="button" role="menuitemradio" aria-checked={active}
            data-tooltip={`主题：切换为${choice.label}`}
            onClick={() => void setTheme(choice.value)}>
            <Icon size={16} />
            <span>{choice.label}</span>
            {active && <Check size={15} />}
          </button>
        );
      })}
    </div>
  );
}
