import { achievementsApi } from "../api/achievements";
import { gamesApi } from "../api/games";
import { journalApi } from "../api/journal";
import { playSessionsApi } from "../api/play-sessions";
import { clear, el } from "../ui/dom";
import { confirmDialog } from "../ui/confirm";
import {
  formatDate,
  formatDateTime,
  formatCalendarPlayPeriod,
  formatRelativeTime,
  formatSessionTimer,
  formatTrackedDuration,
} from "../ui/format";
import {
  ACHIEVEMENT_STATUS_LABELS,
  ACHIEVEMENT_STATUSES,
  type Achievement,
  type AchievementStatus,
  type NewAchievement,
} from "../types/achievement";
import { STATUS_LABELS, type Game } from "../types/game";
import type { JournalEntry } from "../types/journal";
import { openGameForm } from "./game-form";

type AchievementFormInput = Omit<NewAchievement, "displayOrder"> & { displayOrder: number };
type DetailTab = "overview" | "achievements" | "journal" | "details";

const COLLAPSED_TEXT_CHARS = 520;
const COLLAPSED_TEXT_LINES = 6;

const DETAIL_TABS: Array<{ id: DetailTab; label: string }> = [
  { id: "overview", label: "Overview" },
  { id: "achievements", label: "Achievements" },
  { id: "journal", label: "Journal" },
  { id: "details", label: "Details" },
];

/**
 * Game detail: a summary header, user-defined achievements and the game's
 * journal timeline.
 */
export function renderGameDetail(root: HTMLElement, gameId: number, onBack: () => void): void {
  clear(root);
  let activeTab: DetailTab = "overview";
  const container = el("div", {});
  root.append(container);

  const load = async (): Promise<void> => {
    clear(container);
    container.append(el("p", { class: "muted" }, ["Loading…"]));
    try {
      const [game, achievements, entries, activeSession] = await Promise.all([
        gamesApi.get(gameId),
        achievementsApi.listForGame(gameId),
        journalApi.listForGame(gameId),
        playSessionsApi.active(),
      ]);
      clear(container);
      const tabContent = el("div", { class: "detail-tab-content" });
      const renderActiveTab = (): void => {
        clear(tabContent);
        tabContent.append(activeTabContent(activeTab, game, achievements, entries, load));
      };
      container.append(
        header(game, activeSession?.gameId ?? null, onBack, load),
        tabs(activeTab, (tab) => {
          activeTab = tab;
          renderActiveTab();
        }),
        tabContent,
      );
      renderActiveTab();
    } catch (e) {
      clear(container);
      container.append(el("p", { class: "form-error" }, [`Failed to load: ${String(e)}`]));
    }
  };

  void load();
}

function header(
  game: Game,
  activeGameId: number | null,
  onBack: () => void,
  reload: () => Promise<void>,
): HTMLElement {
  const controls: Node[] = [];
  if (game.isPlayingNow) {
    controls.push(playControl(game, reload));
  } else if (activeGameId === null) {
    controls.push(playControl(game, reload));
  }
  return el("div", { class: "view-header" }, [
    el("div", { class: "detail-title" }, [
      el("button", { class: "btn btn-sm", onclick: onBack }, ["← Back"]),
      el("h1", {}, [game.title]),
      ...(game.isPlayingNow ? [el("span", { class: "playing-indicator", title: "Playing now" })] : []),
    ]),
    el("div", { class: "play-controls" }, [...controls, ...headerActions(game, onBack, reload)]),
  ]);
}

function headerActions(game: Game, onBack: () => void, reload: () => Promise<void>): HTMLElement[] {
  return [
    el(
      "button",
      {
        class: "btn btn-sm",
        onclick: () =>
          openGameForm({
            game,
            onSubmit: async (input) => void (await gamesApi.update(game.id, input), await reload()),
          }),
      },
      ["Edit"],
    ),
    el(
      "button",
      {
        class: "btn btn-sm btn-danger",
        onclick: async () => {
          const ok = await confirmDialog(`Delete "${game.title}"? This also deletes its journal.`, {
            danger: true,
            confirmLabel: "Delete",
          });
          if (ok) {
            await gamesApi.delete(game.id);
            onBack();
          }
        },
      },
      ["Delete"],
    ),
  ];
}

function playControl(game: Game, reload: () => Promise<void>): HTMLElement {
  const errorMessage = el("span", { class: "form-error tracking-error" });
  const button = el("button", {
    class: `btn play-toggle ${game.isPlayingNow ? "stop" : "play"}`,
    onclick: async () => {
      button.setAttribute("disabled", "true");
      try {
        if (game.isPlayingNow) await playSessionsApi.stop(game.id);
        else await playSessionsApi.start(game.id);
        await reload();
      } catch (error) {
        button.removeAttribute("disabled");
        errorMessage.textContent = String(error);
      }
    },
  }, [game.isPlayingNow ? "■ Stop" : "▶ Play"]);

  return el("div", { class: "play-control" }, [button, errorMessage]);
}

function tabs(activeTab: DetailTab, onSelect: (tab: DetailTab) => void): HTMLElement {
  const buttons = new Map<DetailTab, HTMLButtonElement>();

  const setActive = (tab: DetailTab): void => {
    for (const [id, button] of buttons) {
      button.classList.toggle("active", id === tab);
    }
    onSelect(tab);
  };

  return el(
    "div",
    { class: "detail-tabs" },
    DETAIL_TABS.map((tab) => {
      const button = el(
        "button",
        {
          class: `detail-tab${tab.id === activeTab ? " active" : ""}`,
          onclick: () => setActive(tab.id),
        },
        [tab.label],
      );
      buttons.set(tab.id, button);
      return button;
    }),
  );
}

function activeTabContent(
  activeTab: DetailTab,
  game: Game,
  achievements: Achievement[],
  entries: JournalEntry[],
  reload: () => Promise<void>,
): HTMLElement {
  switch (activeTab) {
    case "achievements":
      return achievementsSection(game, achievements, reload);
    case "journal":
      return journalSection(game, entries, reload);
    case "details":
      return detailsSection(game);
    case "overview":
      return overviewSection(game, achievements, entries);
  }
}

function overviewSection(game: Game, achievements: Achievement[], entries: JournalEntry[]): HTMLElement {
  const completed = achievements.filter((achievement) => achievement.status === "completed").length;
  const achievementSummary =
    achievements.length === 0 ? "No achievements" : `${completed}/${achievements.length} completed`;
  const latestEntry = entries[0];

  return el("section", { class: "detail-panel overview-panel" }, [
    el("div", { class: "overview-grid" }, [
      overviewCard("Status", STATUS_LABELS[game.status]),
      overviewCard("Play period", formatCalendarPlayPeriod(game.startedAt, game.finishedAt)),
      overviewCard("Total play time", trackedPlayTime(game)),
      ...(game.isPlayingNow && game.activeSessionStartedAt
        ? [liveSessionCard(game.activeSessionStartedAt)]
        : []),
      ...(game.status === "playing" && game.lastPlayedAt
        ? [lastPlayedCard(game.lastPlayedAt)]
        : []),
      overviewCard("Achievements", achievementSummary),
      overviewCard("Journal entries", String(entries.length)),
      overviewCard("Rating", game.rating === null ? "—" : `${game.rating}/10`),
    ]),
    el("div", { class: "overview-columns" }, [
      el("div", { class: "overview-block" }, [
        el("h2", {}, ["Game summary"]),
        gameSummary(game),
      ]),
      el("div", { class: "overview-block" }, [
        el("h2", {}, ["Latest journal entry"]),
        latestEntry
          ? el("div", {}, [
              el("div", { class: "entry-time" }, [formatDateTime(latestEntry.createdAt)]),
              collapsibleText(latestEntry.body, "entry-body"),
            ])
          : el("p", { class: "muted" }, ["No journal entries yet."]),
      ]),
    ]),
  ]);
}

function trackedPlayTime(game: Game): string {
  return game.completedSessionsCount > 0
    ? formatTrackedDuration(game.totalPlayTimeSeconds)
    : "—";
}

function relativeTimeValue(value: string): HTMLElement {
  const element = el("strong", {}, [formatRelativeTime(value)]);
  const interval = window.setInterval(() => {
    if (!element.isConnected) window.clearInterval(interval);
    else element.textContent = formatRelativeTime(value);
  }, 60_000);
  return element;
}

function lastPlayedCard(value: string): HTMLElement {
  return el("div", { class: "overview-card" }, [
    el("span", { class: "muted" }, ["Last played"]),
    relativeTimeValue(value),
  ]);
}

function liveSessionCard(startedAt: string): HTMLElement {
  const value = el("strong", { class: "session-timer" }, [formatSessionTimer(startedAt)]);
  const interval = window.setInterval(() => {
    if (!value.isConnected) window.clearInterval(interval);
    else value.textContent = formatSessionTimer(startedAt);
  }, 1000);
  return el("div", { class: "overview-card live-session-card" }, [
    el("span", { class: "muted" }, ["Current session"]), value,
  ]);
}

function overviewCard(label: string, value: string): HTMLElement {
  return el("div", { class: "overview-card" }, [
    el("span", { class: "muted" }, [label]),
    el("strong", {}, [value]),
  ]);
}

function gameSummary(game: Game): HTMLElement {
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

  return el("div", { class: "game-summary" }, [el("div", { class: "meta-row" }, meta), genres]);
}

function detailsSection(game: Game): HTMLElement {
  const rows: Array<[string, string]> = [
    ["Title", game.title],
    ["Status", STATUS_LABELS[game.status]],
    ["Genres", game.genres.length > 0 ? game.genres.join(", ") : "—"],
    ["Platform", game.platform ?? "—"],
    ["Developer", game.developer ?? "—"],
    ["Publisher", game.publisher ?? "—"],
    ["Release year", game.releaseYear === null ? "—" : String(game.releaseYear)],
    ["Started", formatDate(game.startedAt)],
    ["Finished", formatDate(game.finishedAt)],
    ["Play period", formatCalendarPlayPeriod(game.startedAt, game.finishedAt)],
    ["Rating", game.rating === null ? "—" : `${game.rating}/10`],
    ["Total play time", trackedPlayTime(game)],
    ["Last played", game.lastPlayedAt ? formatRelativeTime(game.lastPlayedAt) : "—"],
    ["Created", formatDateTime(game.createdAt)],
    ["Updated", formatDateTime(game.updatedAt)],
  ];

  return el("section", { class: "detail-panel" }, [
    el("div", { class: "section-head" }, [el("div", {}, [el("h2", {}, ["Details"])])]),
    el(
      "dl",
      { class: "details-list" },
      rows.flatMap(([label, value]) => [
        el("dt", {}, [label]),
        el("dd", {}, [value]),
      ]),
    ),
  ]);
}

function metaItem(label: string, value: string): HTMLElement {
  return el("span", { class: "meta-item" }, [
    el("span", { class: "muted" }, [`${label}: `]),
    el("span", {}, [value]),
  ]);
}

function collapsibleText(text: string, className: string): HTMLElement {
  if (!shouldCollapseText(text)) {
    return el("p", { class: className }, [text]);
  }

  let expanded = false;
  const body = el("p", { class: className }, [collapsedText(text)]);
  const toggle = el(
    "button",
    {
      class: "link show-more",
      onclick: () => {
        expanded = !expanded;
        body.textContent = expanded ? text : collapsedText(text);
        toggle.textContent = expanded ? "Show less" : "Show all";
      },
    },
    ["Show all"],
  );

  return el("div", { class: "collapsible-text" }, [body, toggle]);
}

function shouldCollapseText(text: string): boolean {
  return text.length > COLLAPSED_TEXT_CHARS || text.split(/\r?\n/).length > COLLAPSED_TEXT_LINES;
}

function collapsedText(text: string): string {
  const lines = text.split(/\r?\n/);
  const linePreview =
    lines.length > COLLAPSED_TEXT_LINES ? lines.slice(0, COLLAPSED_TEXT_LINES).join("\n") : text;

  if (linePreview.length <= COLLAPSED_TEXT_CHARS) {
    return `${linePreview.trimEnd()}...`;
  }

  const preview = linePreview.slice(0, COLLAPSED_TEXT_CHARS).trimEnd();
  const lastSpace = preview.lastIndexOf(" ");
  const trimmed = lastSpace > COLLAPSED_TEXT_CHARS * 0.65 ? preview.slice(0, lastSpace) : preview;
  return `${trimmed.trimEnd()}...`;
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

  return el("section", { class: "detail-panel achievements-panel" }, [
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
  const percent = achievementProgressPercent(achievement);

  return el("article", { class: `achievement-card achievement-${achievement.status}` }, [
    el("div", { class: "achievement-card-head" }, [
      el("div", { class: "achievement-title-row" }, [
        el("h4", {}, [achievement.isHidden ? `Hidden: ${achievement.title}` : achievement.title]),
        el("span", { class: `badge achievement-badge achievement-badge-${achievement.status}` }, [
          ACHIEVEMENT_STATUS_LABELS[achievement.status],
        ]),
      ]),
      achievementActions(game, achievement, reload),
    ]),
    ...(percent !== null ? [progressRing(percent)] : []),
    ...(achievement.description ? [collapsibleText(achievement.description, "achievement-description")] : []),
    ...(progress ? [progress] : []),
    ...(completedAt ? [completedAt] : [el("span", { class: "muted" }, ["—"])]),
  ]);
}

function achievementActions(
  game: Game,
  achievement: Achievement,
  reload: () => Promise<void>,
): HTMLElement {
  return el("div", { class: "achievement-actions" }, [
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
  ]);
}

function achievementProgressPercent(achievement: Achievement): number | null {
  if (achievement.progressCurrent === null || achievement.progressTarget === null) {
    return null;
  }
  return Math.min(100, Math.round((achievement.progressCurrent / achievement.progressTarget) * 100));
}

function progressRing(percent: number): HTMLElement {
  return el(
    "div",
    {
      class: "progress-ring",
      style: `--progress: ${percent}%`,
    },
    [el("span", {}, [`${percent}%`])],
  );
}

function progressBlock(achievement: Achievement): HTMLElement {
  const current = achievement.progressCurrent ?? 0;
  const target = achievement.progressTarget ?? 1;
  const unit = achievement.progressUnit ? ` ${achievement.progressUnit}` : "";

  return el("div", { class: "achievement-progress" }, [
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

  return el("section", { class: "detail-panel journal" }, [
    el("div", { class: "section-head" }, [el("div", {}, [el("h2", {}, ["Journal"])])]),
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
      collapsibleText(entry.body, "entry-body"),
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
