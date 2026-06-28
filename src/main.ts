import "./styles.css";
import { el } from "./ui/dom";
import { renderGamesView } from "./views/games-view";

/**
 * Application shell: a fixed sidebar with navigation and a content area. Each nav
 * entry maps to a view renderer. Future modules (Timeline, Achievements, Stats…)
 * register here without touching existing views.
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
  { id: "stats", label: "Statistics", render: placeholder("Statistics"), enabled: false },
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

  const select = (item: NavItem): void => {
    for (const [id, btn] of buttons) btn.classList.toggle("active", id === item.id);
    item.render(content);
  };

  const nav = el(
    "nav",
    { class: "sidebar" },
    [
      el("div", { class: "brand" }, ["Xareon"]),
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
}

mount();
