import { gamesApi } from "../api/games";
import { clear, el } from "../ui/dom";
import { STATUS_LABELS, type Game } from "../types/game";
import { openGameForm } from "./game-form";

/** Renders the games library: a header with an "Add" action and a table of games. */
export function renderGamesView(root: HTMLElement): void {
  const list = el("div", { class: "view-body" });

  const reload = async (): Promise<void> => {
    clear(list);
    list.append(el("p", { class: "muted" }, ["Loading…"]));
    try {
      const games = await gamesApi.list();
      clear(list);
      list.append(games.length === 0 ? emptyState() : gamesTable(games, reload));
    } catch (e) {
      clear(list);
      list.append(el("p", { class: "form-error" }, [`Failed to load games: ${String(e)}`]));
    }
  };

  const header = el("div", { class: "view-header" }, [
    el("h1", {}, ["Games"]),
    el(
      "button",
      {
        class: "btn btn-primary",
        onclick: () => openGameForm({ game: null, onSubmit: async (i) => void (await gamesApi.create(i), await reload()) }),
      },
      ["+ Add game"],
    ),
  ]);

  clear(root);
  root.append(header, list);
  void reload();
}

function emptyState(): HTMLElement {
  return el("div", { class: "empty" }, [
    el("p", {}, ["No games yet."]),
    el("p", { class: "muted" }, ["Add the first game to start your journal."]),
  ]);
}

function gamesTable(games: Game[], reload: () => Promise<void>): HTMLElement {
  const rows = games.map((game) =>
    el("tr", {}, [
      el("td", {}, [game.title]),
      el("td", {}, [game.platform ?? "—"]),
      el("td", {}, [el("span", { class: `badge status-${game.status}` }, [STATUS_LABELS[game.status]])]),
      el("td", { class: "num" }, [game.rating === null ? "—" : `${game.rating}/10`]),
      el("td", { class: "actions" }, [
        el(
          "button",
          {
            class: "btn btn-sm",
            onclick: () =>
              openGameForm({
                game,
                onSubmit: async (i) => void (await gamesApi.update(game.id, i), await reload()),
              }),
          },
          ["Edit"],
        ),
        el(
          "button",
          {
            class: "btn btn-sm btn-danger",
            onclick: async () => {
              if (confirm(`Delete "${game.title}"?`)) {
                await gamesApi.delete(game.id);
                await reload();
              }
            },
          },
          ["Delete"],
        ),
      ]),
    ]),
  );

  return el("table", { class: "data-table" }, [
    el("thead", {}, [
      el("tr", {}, [
        el("th", {}, ["Title"]),
        el("th", {}, ["Platform"]),
        el("th", {}, ["Status"]),
        el("th", { class: "num" }, ["Rating"]),
        el("th", {}, [""]),
      ]),
    ]),
    el("tbody", {}, rows),
  ]);
}
