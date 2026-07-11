import "./styles.css";
import { listen } from "@tauri-apps/api/event";
import { el } from "./ui/dom";
import { renderGamesView } from "./views/games-view";
import { renderStatisticsView } from "./views/statistics-view";
import { renderSettingsView } from "./views/settings-view";

/**
 * Application shell: a fixed sidebar with navigation and a content area. Each nav
 * entry maps to a view renderer. Future global modules (Timeline, Stats…) register
 * here without touching existing views.
 */

interface NavItem {
  id: string;
  label: string;
  render: (root: HTMLElement) => void;
  enabled: boolean;
}

const NAV: NavItem[] = [
  { id: "games", label: "Games", render: renderGamesView, enabled: true },
  { id: "timeline", label: "Timeline", render: placeholder("Timeline"), enabled: false },
  { id: "achievements", label: "Achievements", render: placeholder("Achievements"), enabled: false },
  { id: "stats", label: "Statistics", render: renderStatisticsView, enabled: true },
  { id: "settings", label: "Settings", render: renderSettingsView, enabled: true },
];

function placeholder(name: string): (root: HTMLElement) => void {
  return (root) => {
    root.replaceChildren(
      el("div", { class: "view-header" }, [el("h1", {}, [name])]),
      el("p", { class: "muted" }, ["Coming soon."]),
    );
  };
}

function mount(): void {
  const app = document.getElementById("app");
  if (!app) throw new Error("#app root not found");

  const content = el("main", { class: "content" });
  const buttons = new Map<string, HTMLButtonElement>();
  let current: NavItem | undefined;

  const select = (item: NavItem): void => {
    current = item;
    for (const [id, btn] of buttons) btn.classList.toggle("active", id === item.id);
    item.render(content);
  };

  const nav = el(
    "nav",
    { class: "sidebar" },
    [
      el("div", { class: "brand" }, [
        el("img", { class: "brand-icon", src: "/xareon-icon.png", alt: "" }),
        el("span", {}, ["Xareon"]),
      ]),
      ...NAV.map((item) => {
        const btn = el(
          "button",
          {
            class: "nav-item",
            disabled: !item.enabled,
            onclick: item.enabled ? () => select(item) : null,
          },
          [item.label],
        );
        buttons.set(item.id, btn);
        return btn;
      }),
    ],
  );

  app.append(el("div", { class: "layout" }, [nav, content]));

  const first = NAV.find((n) => n.enabled);
  if (first) select(first);

  const toast = el("div", { class: "tracking-toast hidden" });
  app.append(toast);
  void listen<{ gameId: number | null; isPlaying: boolean; error: string | null }>(
    "play-tracking-changed",
    ({ payload }) => {
      if (current) current.render(content);
      toast.textContent = payload.error ?? (payload.isPlaying ? "Play tracking started" : "Play tracking stopped");
      toast.classList.remove("hidden", "error");
      toast.classList.toggle("error", payload.error !== null);
      window.setTimeout(() => toast.classList.add("hidden"), 3000);
    },
  );
}

mount();
