import { achievementsApi } from "../api/achievements";
import { gamesApi } from "../api/games";
import { journalApi } from "../api/journal";
import { clear, el } from "../ui/dom";
import { confirmDialog } from "../ui/confirm";
import { formatDate, formatDateTime } from "../ui/format";
import {
  ACHIEVEMENT_STATUS_LABELS,
  ACHIEVEMENT_STATUSES,
  type Achievement,
  type AchievementStatus,
  type NewAchievement,
} from "../types/achievement";
import { STATUS_LABELS, type Game } from "../types/game";
import type { JournalEntry } from "../types/journal";

type AchievementFormInput = Omit<NewAchievement, "displayOrder"> & { displayOrder: number };

/**
 * Game detail: a summary header, user-defined achievements and the game's
 * journal timeline.
 */
export function renderGameDetail(root: HTMLElement, gameId: number, onBack: () => void): void {
  clear(root);
  const container = el("div", {});
  root.append(container);

  const load = async (): Promise<void> => {
    clear(container);
    container.append(el("p", { class: "muted" }, ["Loading…"]));
    try {
      const [game, achievements, entries] = await Promise.all([
        gamesApi.get(gameId),
        achievementsApi.listForGame(gameId),
        journalApi.listForGame(gameId),
      ]);
      clear(container);
      container.append(
        header(game, onBack),
        summary(game),
        achievementsSection(game, achievements, load),
        journalSection(game, entries, load),
      );
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

function achievementsSection(
  game: Game,
  achievements: Achievement[],
  reload: () => Promise<void>,
): HTMLElement {
  const completed = achievements.filter((achievement) => achievement.status === "completed").length;
  const hidden = achievements.filter((achievement) => achievement.isHidden).length;
  const stats = [
    `${completed}/${achievements.length} completed`,
    ...(hidden > 0 ? [`${hidden} hidden`] : []),
  ].join(" · ");

  const add = el(
    "button",
    {
      class: "btn btn-primary",
      onclick: () =>
        openAchievementForm({
          game,
          achievement: null,
          nextOrder: achievements.length,
          onSubmit: async (input) => {
            await achievementsApi.create(input);
            await reload();
          },
        }),
    },
    ["Add achievement"],
  );

  const content =
    achievements.length === 0
      ? el("p", { class: "muted" }, ["No achievements yet. Add personal milestones for this game."])
      : el(
          "div",
          { class: "achievement-groups" },
          groupAchievements(achievements).map(([category, items]) =>
            el("div", { class: "achievement-group" }, [
              el("h3", {}, [category]),
              el(
                "div",
                { class: "achievement-list" },
                items.map((achievement) => achievementCard(game, achievement, reload)),
              ),
            ]),
          ),
        );

  return el("section", { class: "achievements" }, [
    el("div", { class: "section-head" }, [
      el("div", {}, [el("h2", {}, ["Achievements"]), el("p", { class: "muted" }, [stats])]),
      add,
    ]),
    content,
  ]);
}

function groupAchievements(achievements: Achievement[]): Array<[string, Achievement[]]> {
  const groups = new Map<string, Achievement[]>();
  for (const achievement of achievements) {
    const category = achievement.category ?? "General";
    groups.set(category, [...(groups.get(category) ?? []), achievement]);
  }
  return [...groups.entries()];
}

function achievementCard(
  game: Game,
  achievement: Achievement,
  reload: () => Promise<void>,
): HTMLElement {
  const progress =
    achievement.progressCurrent !== null && achievement.progressTarget !== null
      ? progressBlock(achievement)
      : null;
  const completedAt =
    achievement.completedAt && achievement.status === "completed"
      ? el("span", { class: "muted" }, [`Completed ${formatDate(achievement.completedAt)}`])
      : null;

  return el("article", { class: `achievement achievement-${achievement.status}` }, [
    el("div", { class: "achievement-main" }, [
      el("div", { class: "achievement-title-row" }, [
        el("h4", {}, [achievement.isHidden ? `Hidden: ${achievement.title}` : achievement.title]),
        el("span", { class: `badge achievement-badge achievement-badge-${achievement.status}` }, [
          ACHIEVEMENT_STATUS_LABELS[achievement.status],
        ]),
      ]),
      ...(achievement.description
        ? [el("p", { class: "achievement-description" }, [achievement.description])]
        : []),
      ...(progress ? [progress] : []),
      ...(completedAt ? [completedAt] : []),
    ]),
    el("div", { class: "achievement-actions" }, [
      achievement.status === "completed"
        ? el(
            "button",
            {
              class: "btn btn-sm",
              onclick: async () => {
                await achievementsApi.reopen(achievement.id);
                await reload();
              },
            },
            ["Reopen"],
          )
        : el(
            "button",
            {
              class: "btn btn-sm btn-primary",
              onclick: async () => {
                await achievementsApi.complete(achievement.id);
                await reload();
              },
            },
            ["Complete"],
          ),
      el(
        "button",
        {
          class: "btn btn-sm",
          onclick: () =>
            openAchievementForm({
              game,
              achievement,
              nextOrder: achievement.displayOrder,
              onSubmit: async (input) => {
                await achievementsApi.update(achievement.id, input);
                await reload();
              },
            }),
        },
        ["Edit"],
      ),
      el(
        "button",
        {
          class: "btn btn-sm btn-danger",
          onclick: async () => {
            if (await confirmDialog("Delete this achievement?", { danger: true, confirmLabel: "Delete" })) {
              await achievementsApi.delete(achievement.id);
              await reload();
            }
          },
        },
        ["Delete"],
      ),
    ]),
  ]);
}

function progressBlock(achievement: Achievement): HTMLElement {
  const current = achievement.progressCurrent ?? 0;
  const target = achievement.progressTarget ?? 1;
  const percent = Math.min(100, Math.round((current / target) * 100));
  const unit = achievement.progressUnit ? ` ${achievement.progressUnit}` : "";

  return el("div", { class: "achievement-progress" }, [
    el("div", { class: "progress-track" }, [el("div", { class: "progress-fill", style: `width: ${percent}%` })]),
    el("span", { class: "muted" }, [`${current}/${target}${unit}`]),
  ]);
}

function openAchievementForm(options: {
  game: Game;
  achievement: Achievement | null;
  nextOrder: number;
  onSubmit: (input: AchievementFormInput) => Promise<void>;
}): void {
  const { game, achievement, nextOrder, onSubmit } = options;

  const textField = (
    name: string,
    label: string,
    value: string | number | null,
    type = "text",
  ): { row: HTMLElement; input: HTMLInputElement } => {
    const input = el("input", { name, type, value: value === null ? "" : String(value) });
    return { row: el("label", { class: "field" }, [el("span", {}, [label]), input]), input };
  };

  const title = textField("title", "Title", achievement?.title ?? "");
  const category = textField("category", "Category", achievement?.category ?? "");
  const progressCurrent = textField(
    "progressCurrent",
    "Progress current",
    achievement?.progressCurrent ?? "",
    "number",
  );
  const progressTarget = textField(
    "progressTarget",
    "Progress target",
    achievement?.progressTarget ?? "",
    "number",
  );
  const progressUnit = textField("progressUnit", "Progress unit", achievement?.progressUnit ?? "");
  const completedAt = textField(
    "completedAt",
    "Completed date",
    achievement?.completedAt ? achievement.completedAt.slice(0, 10) : "",
    "date",
  );
  const displayOrder = textField("displayOrder", "Display order", achievement?.displayOrder ?? nextOrder, "number");
  const description = el("textarea", {
    name: "description",
    rows: 4,
    placeholder: "Optional note",
  }, [achievement?.description ?? ""]);
  const descriptionRow = el("label", { class: "field field-wide" }, [
    el("span", {}, ["Description"]),
    description,
  ]);
  const status = el(
    "select",
    { name: "status" },
    ACHIEVEMENT_STATUSES.map((value) =>
      el("option", { value, selected: (achievement?.status ?? "planned") === value }, [
        ACHIEVEMENT_STATUS_LABELS[value],
      ]),
    ),
  );
  const statusRow = el("label", { class: "field" }, [el("span", {}, ["Status"]), status]);
  const hidden = el("input", { name: "isHidden", type: "checkbox", checked: achievement?.isHidden ?? false });
  const hiddenRow = el("label", { class: "field checkbox-field" }, [
    hidden,
    el("span", {}, ["Hidden / non-obvious"]),
  ]);
  const error = el("p", { class: "form-error" });

  const overlay = el("div", { class: "modal-overlay" });
  const close = (): void => overlay.remove();
  const text = (input: HTMLInputElement | HTMLTextAreaElement): string | null => {
    const value = input.value.trim();
    return value === "" ? null : value;
  };
  const int = (input: HTMLInputElement): number | null => {
    const value = input.value.trim();
    if (value === "") return null;
    const number = Number(value);
    return Number.isFinite(number) ? Math.trunc(number) : null;
  };

  const form = el(
    "form",
    {
      class: "modal",
      onsubmit: async (event: Event) => {
        event.preventDefault();
        const titleValue = text(title.input);
        if (titleValue === null) {
          error.textContent = "Title is required.";
          return;
        }

        const input: AchievementFormInput = {
          gameId: game.id,
          title: titleValue,
          description: text(description),
          category: text(category.input),
          status: status.value as AchievementStatus,
          progressCurrent: int(progressCurrent.input),
          progressTarget: int(progressTarget.input),
          progressUnit: text(progressUnit.input),
          completedAt: text(completedAt.input),
          isHidden: hidden.checked,
          displayOrder: int(displayOrder.input) ?? 0,
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
      el("h2", {}, [achievement ? "Edit achievement" : "Add achievement"]),
      el("div", { class: "form-grid" }, [
        title.row,
        statusRow,
        category.row,
        displayOrder.row,
        progressCurrent.row,
        progressTarget.row,
        progressUnit.row,
        completedAt.row,
        hiddenRow,
        descriptionRow,
      ]),
      error,
      el("div", { class: "modal-actions" }, [
        el("button", { type: "button", class: "btn", onclick: close }, ["Cancel"]),
        el("button", { type: "submit", class: "btn btn-primary" }, ["Save"]),
      ]),
    ],
  );

  overlay.append(form);
  document.body.append(overlay);
  title.input.focus();
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
                if (await confirmDialog("Delete this entry?", { danger: true, confirmLabel: "Delete" })) {
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
