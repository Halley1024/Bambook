import { ChevronDown, type LucideIcon } from "lucide-react";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";

export type DropdownMenuItem = {
  id: string;
  label: string;
  icon?: LucideIcon;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  title?: string;
  onSelect?: () => void;
  submenu?: ReactNode;
};

export type DropdownMenuGroup = {
  id: string;
  items: DropdownMenuItem[];
};

export function DropdownMenu({ label, icon: TriggerIcon, groups }: {
  label: string;
  icon: LucideIcon;
  groups: DropdownMenuGroup[];
}) {
  const [open, setOpen] = useState(false);
  const hostRef = useRef<HTMLDivElement | null>(null);
  const menuId = useId();

  useEffect(() => {
    if (!open) return;
    const closeOutside = (event: PointerEvent) => {
      if (!hostRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        hostRef.current?.querySelector<HTMLButtonElement>(".app-menu-trigger")?.focus();
      }
    };
    window.addEventListener("pointerdown", closeOutside);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("pointerdown", closeOutside);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  return (
    <div className="app-menu-host" ref={hostRef}>
      <button
        className={`tool-button app-menu-trigger ${open ? "active" : ""}`}
        data-tooltip={`${label}：打开${label}菜单`}
        type="button"
        aria-haspopup="menu"
        aria-controls={menuId}
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <TriggerIcon size={17} aria-hidden="true" />
        <span>{label}</span>
        <ChevronDown className="app-menu-chevron" size={14} aria-hidden="true" />
      </button>

      {open && (
        <div className="app-dropdown-menu" id={menuId} role="menu" aria-label={`${label}菜单`}>
          {groups.map((group, groupIndex) => (
            <div className="app-dropdown-menu-group" key={group.id} role="group">
              {groupIndex > 0 && <div className="app-dropdown-menu-divider" role="separator" />}
              {group.items.map((item) => {
                const ItemIcon = item.icon;
                return (
                  <div className="app-dropdown-menu-item-host" key={item.id}>
                  <button
                    className={`app-dropdown-menu-item ${item.danger ? "danger" : ""}`}
                    type="button"
                    role="menuitem"
                    disabled={item.disabled}
                    data-tooltip={item.title ?? `选择：${item.label}`}
                    onClick={() => {
                      if (!item.submenu) {
                        item.onSelect?.();
                        setOpen(false);
                      }
                    }}
                  >
                    <span className="app-dropdown-menu-icon">{ItemIcon && <ItemIcon size={16} />}</span>
                    <span className="app-dropdown-menu-label">{item.label}</span>
                    {item.shortcut && <span className="app-dropdown-menu-shortcut">{item.shortcut}</span>}
                  </button>
                  {item.submenu && <div className="app-dropdown-submenu">{item.submenu}</div>}
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
