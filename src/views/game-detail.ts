import { gamesApi } from "../api/games";
import { journalApi } from "../api/journal";
import { clear, el } from "../ui/dom";
import { formatDate, formatDateTime } from "../ui/format";
import { STATUS_LABELS, type Game } from "../types/game";
import type { JournalEntry } from "../types/journal";

/**
 * Game detail: a summary header plus the game's journal, shown as a reverse-
 * chronological timeline of memories with quick add/edit/delete.
 */
export function renderGameDetail(root: HTMLElement, gameId: number, onBack: () => void): void {
  clear(root);
  const container = el("div", {});
  root.append(container);

  const load = async (): Promise<void> => {
    clear(container);
    container.append(el("p", { class: "muted" }, ["Loading…"]));
    try {
      const [game, entries] = await Promise.all([
        gamesApi.get(gameId),
        journalApi.listForGame(gameId),
      ]);
      clear(container);
      container.append(header(game, onBack), summary(game), journalSection(game, entries, load));
    } catch (e) {
      clear(container);
      container.append(el("p", { class: "form-error" }, [`Failed to load: ${String(e)}`]));
    }
  };

  void load();
}

function header(game: Game, onBack: () => void): HTMLElement {
  return el("div", { class: "view-header" }, [
    el("div", { class: "detail-title" }, [
      el("button", { class: "btn btn-sm", onclick: onBack }, ["← Back"]),
      el("h1", {}, [game.title]),
    ]),
  ]);
}

function summary(game: Game): HTMLElement {
  const meta: Array<Node> = [
    el("span", { class: `badge status-${game.status}` }, [STATUS_LABELS[game.status]]),
  ];
  if (game.rating !== null) meta.push(metaItem("Rating", `${game.rating}/10`));
  if (game.platform) meta.push(metaItem("Platform", game.platform));
  if (game.releaseYear !== null) meta.push(metaItem("Released", String(game.releaseYear)));
  if (game.developer) meta.push(metaItem("Developer", game.developer));
  if (game.publisher) meta.push(metaItem("Publisher", game.publisher));
  meta.push(metaItem("Started", formatDate(game.startedAt)));
  meta.push(metaItem("Finished", formatDate(game.finishedAt)));

  const genres =
    game.genres.length > 0
      ? el(
          "div",
          { class: "genre-chips" },
          game.genres.map((g) => el("span", { class: "chip static" }, [g])),
        )
      : el("span", { class: "muted" }, ["No genres"]);

  return el("div", { class: "detail-summary" }, [el("div", { class: "meta-row" }, meta), genres]);
}

function metaItem(label: string, value: string): HTMLElement {
  return el("span", { class: "meta-item" }, [
    el("span", { class: "muted" }, [`${label}: `]),
    el("span", {}, [value]),
  ]);
}

function journalSection(
  game: Game,
  entries: JournalEntry[],
  reload: () => Promise<void>,
): HTMLElement {
  const composer = el("textarea", {
    class: "composer",
    rows: 3,
    placeholder: "Write a journal entry…",
  });
  const error = el("p", { class: "form-error" });

  const save = el(
    "button",
    {
      class: "btn btn-primary",
      onclick: async () => {
        const body = composer.value.trim();
        if (!body) {
          error.textContent = "Entry cannot be empty.";
          return;
        }
        try {
          await journalApi.create({ gameId: game.id, body });
          composer.value = "";
          error.textContent = "";
          await reload();
        } catch (e) {
          error.textContent = String(e);
        }
      },
    },
    ["Add entry"],
  );

  const timeline =
    entries.length === 0
      ? el("p", { class: "muted" }, ["No entries yet. Start your journal above."])
      : el("div", { class: "timeline" }, entries.map((entry) => entryCard(entry, reload)));

  return el("section", { class: "journal" }, [
    el("h2", {}, ["Journal"]),
    el("div", { class: "composer-box" }, [composer, error, el("div", { class: "composer-actions" }, [save])]),
    timeline,
  ]);
}

function entryCard(entry: JournalEntry, reload: () => Promise<void>): HTMLElement {
  const card = el("article", { class: "entry" });

  const edited = entry.updatedAt !== entry.createdAt;
  const stamp = el("div", { class: "entry-time" }, [
    formatDateTime(entry.createdAt),
    ...(edited ? [el("span", { class: "muted" }, [` · edited ${formatDateTime(entry.updatedAt)}`])] : []),
  ]);

  const renderRead = (): void => {
    clear(card);
    card.append(
      el("div", { class: "entry-head" }, [
        stamp,
        el("div", { class: "entry-actions" }, [
          el("button", { class: "btn btn-sm", onclick: renderEdit }, ["Edit"]),
          el(
            "button",
            {
              class: "btn btn-sm btn-danger",
              onclick: async () => {
                if (confirm("Delete this entry?")) {
                  await journalApi.delete(entry.id);
                  await reload();
                }
              },
            },
            ["Delete"],
          ),
        ]),
      ]),
      el("p", { class: "entry-body" }, [entry.body]),
    );
  };

  const renderEdit = (): void => {
    clear(card);
    const textarea = el("textarea", { class: "composer", rows: 4 }, [entry.body]);
    const error = el("p", { class: "form-error" });
    card.append(
      el("div", { class: "entry-head" }, [stamp]),
      textarea,
      error,
      el("div", { class: "entry-actions" }, [
        el("button", { class: "btn btn-sm", onclick: renderRead }, ["Cancel"]),
        el(
          "button",
          {
            class: "btn btn-sm btn-primary",
            onclick: async () => {
              const body = textarea.value.trim();
              if (!body) {
                error.textContent = "Entry cannot be empty.";
                return;
              }
              try {
                await journalApi.update(entry.id, { body });
                await reload();
              } catch (e) {
                error.textContent = String(e);
              }
            },
          },
          ["Save"],
        ),
      ]),
    );
    textarea.focus();
  };

  renderRead();
  return card;
}
