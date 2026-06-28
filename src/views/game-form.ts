import { genresApi } from "../api/games";
import { el } from "../ui/dom";
import { GAME_STATUSES, STATUS_LABELS, type Game, type GameInput } from "../types/game";

/** Split a comma-separated genre input into trimmed, non-empty names. */
function parseGenres(value: string): string[] {
  return value
    .split(",")
    .map((g) => g.trim())
    .filter((g) => g.length > 0);
}

/** A modal form for creating or editing a game. Returns the collected input via `onSubmit`. */
export function openGameForm(options: {
  game: Game | null;
  onSubmit: (input: GameInput) => Promise<void>;
}): void {
  const { game, onSubmit } = options;

  const field = (
    name: string,
    label: string,
    value: string | number | null,
    type = "text",
  ): { row: HTMLElement; input: HTMLInputElement } => {
    const input = el("input", {
      name,
      type,
      value: value === null ? "" : String(value),
    });
    const row = el("label", { class: "field" }, [el("span", {}, [label]), input]);
    return { row, input };
  };

  const title = field("title", "Title", game?.title ?? "");
  const platform = field("platform", "Platform", game?.platform ?? "");
  const developer = field("developer", "Developer", game?.developer ?? "");
  const publisher = field("publisher", "Publisher", game?.publisher ?? "");
  const releaseYear = field("releaseYear", "Release year", game?.releaseYear ?? "", "number");
  const startedAt = field("startedAt", "Started", game?.startedAt ?? "", "date");
  const finishedAt = field("finishedAt", "Finished", game?.finishedAt ?? "", "date");
  const rating = field("rating", "Rating (0–10)", game?.rating ?? "", "number");

  // Genres: free-text, comma-separated, with suggestions from existing genres.
  const genresList = el("datalist", { id: "genre-suggestions" });
  const genresInput = el("input", {
    name: "genres",
    value: (game?.genres ?? []).join(", "),
    placeholder: "Action, RPG, Adventure",
    list: "genre-suggestions",
    autocomplete: "off",
  });
  const genresRow = el("label", { class: "field field-wide" }, [
    el("span", {}, ["Genres (comma-separated)"]),
    genresInput,
    genresList,
  ]);
  void genresApi.list().then((genres) => {
    genresList.replaceChildren(...genres.map((g) => el("option", { value: g.name })));
  });

  const statusSelect = el(
    "select",
    { name: "status" },
    GAME_STATUSES.map((s) =>
      el("option", { value: s, selected: (game?.status ?? "planned") === s }, [STATUS_LABELS[s]]),
    ),
  );
  const statusRow = el("label", { class: "field" }, [el("span", {}, ["Status"]), statusSelect]);

  const error = el("p", { class: "form-error" });

  const text = (input: HTMLInputElement): string | null => {
    const v = input.value.trim();
    return v === "" ? null : v;
  };
  const int = (input: HTMLInputElement): number | null => {
    const v = input.value.trim();
    if (v === "") return null;
    const n = Number(v);
    return Number.isFinite(n) ? Math.trunc(n) : null;
  };

  const overlay = el("div", { class: "modal-overlay" });
  const close = (): void => overlay.remove();

  const form = el(
    "form",
    {
      class: "modal",
      onsubmit: async (event: Event) => {
        event.preventDefault();
        if (text(title.input) === null) {
          error.textContent = "Title is required.";
          return;
        }
        const input: GameInput = {
          title: title.input.value.trim(),
          genres: parseGenres(genresInput.value),
          platform: text(platform.input),
          developer: text(developer.input),
          publisher: text(publisher.input),
          releaseYear: int(releaseYear.input),
          startedAt: text(startedAt.input),
          finishedAt: text(finishedAt.input),
          status: statusSelect.value as GameInput["status"],
          rating: int(rating.input),
          coverPath: game?.coverPath ?? null,
        };
        try {
          await onSubmit(input);
          close();
        } catch (e) {
          error.textContent = String(e);
        }
      },
    },
    [
      el("h2", {}, [game ? "Edit game" : "Add game"]),
      el("div", { class: "form-grid" }, [
        title.row,
        statusRow,
        genresRow,
        platform.row,
        developer.row,
        publisher.row,
        releaseYear.row,
        rating.row,
        startedAt.row,
        finishedAt.row,
      ]),
      error,
      el("div", { class: "modal-actions" }, [
        el("button", { type: "button", class: "btn", onclick: close }, ["Cancel"]),
        el("button", { type: "submit", class: "btn btn-primary" }, ["Save"]),
      ]),
    ],
  );

  // Intentionally no "click outside to close": the form holds unsaved input, so
  // it closes only via Cancel or a successful save to prevent accidental data loss.
  overlay.append(form);
  document.body.append(overlay);
  title.input.focus();
}
