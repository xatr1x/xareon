import { settingsApi } from "../api/settings";
import { clear, el } from "../ui/dom";
import { confirmDialog } from "../ui/confirm";
import type { ProfileSyncInfo, ProfileSyncStatus, Settings } from "../types/settings";

/**
 * Settings page. Loads the current values on open and persists them on Save.
 * Built as a flat list of fields so adding a setting later is one more `field`
 * entry plus its mapping in `collect` — no structural change.
 */
export function renderSettingsView(root: HTMLElement): void {
  clear(root);
  const container = el("div", {});
  root.append(container);

  const load = async (): Promise<void> => {
    clear(container);
    container.append(el("p", { class: "muted" }, ["Loading…"]));
    try {
      const [settings, syncInfo] = await Promise.all([
        settingsApi.get(),
        settingsApi.getProfileSyncInfo(),
      ]);
      clear(container);
      container.append(form(settings, syncInfo, load));
    } catch (e) {
      clear(container);
      container.append(el("p", { class: "form-error" }, [`Failed to load: ${String(e)}`]));
    }
  };

  void load();
}

function form(settings: Settings, syncInfo: ProfileSyncInfo, reload: () => Promise<void>): HTMLElement {
  const field = (
    name: string,
    label: string,
    value: string | null,
    hint: string,
  ): { row: HTMLElement; input: HTMLInputElement } => {
    const input = el("input", { name, type: "text", value: value ?? "" });
    const row = el("label", { class: "field field-wide" }, [
      el("span", {}, [label]),
      input,
      el("span", { class: "field-hint" }, [hint]),
    ]);
    return { row, input };
  };

  const userIdentifier = field(
    "userIdentifier",
    "User identifier",
    settings.userIdentifier,
    "Your public, human-readable handle in Xareon (e.g. vitalii). Shared with friends. Not a UUID.",
  );
  const playTrackingShortcut = shortcutField(settings.playTrackingShortcut);

  const message = el("p", { class: "form-success" });
  const error = el("p", { class: "form-error" });

  const text = (input: HTMLInputElement): string | null => {
    const v = input.value.trim();
    return v === "" ? null : v;
  };

  return el(
    "form",
    {
      class: "settings-form",
      onsubmit: async (event: Event) => {
        event.preventDefault();
        message.textContent = "";
        error.textContent = "";
        const input: Settings = {
          userIdentifier: text(userIdentifier.input),
          playTrackingShortcut: playTrackingShortcut.value(),
        };
        try {
          const saved = await settingsApi.update(input);
          userIdentifier.input.value = saved.userIdentifier ?? "";
          playTrackingShortcut.set(saved.playTrackingShortcut);
          message.textContent = "Settings saved.";
        } catch (e) {
          error.textContent = String(e);
        }
      },
    },
    [
      el("div", { class: "view-header" }, [el("h1", {}, ["Settings"])]),
      el("div", { class: "form-grid" }, [
        userIdentifier.row,
        playTrackingShortcut.row,
      ]),
      syncSection(syncInfo, reload, error, message),
      error,
      message,
      el("div", { class: "modal-actions" }, [
        el("button", { type: "submit", class: "btn btn-primary" }, ["Save"]),
      ]),
    ],
  );
}

function syncSection(
  info: ProfileSyncInfo,
  reload: () => Promise<void>,
  error: HTMLElement,
  message: HTMLElement,
): HTMLElement {
  const busy = (value: boolean, buttons: HTMLButtonElement[]): void => {
    for (const button of buttons) button.disabled = value;
  };
  const choose = el("button", { type: "button", class: "btn" }, [
    info.folder ? "Change folder" : "Choose folder",
  ]);
  const openDb = el("button", { type: "button", class: "btn" }, ["Open DB folder"]);
  const upload = el("button", { type: "button", class: "btn btn-primary" }, ["Upload backup"]);
  const restore = el("button", { type: "button", class: "btn btn-danger" }, ["Download & restore"]);
  const buttons = [choose, openDb, upload, restore];
  upload.disabled = !info.folder;
  restore.disabled = !info.folder || info.status === "backupUnavailable" || info.status === "invalidBackup";

  const run = async (action: () => Promise<unknown>, success?: string): Promise<void> => {
    error.textContent = "";
    message.textContent = "";
    busy(true, buttons);
    try {
      await action();
      if (success) message.textContent = success;
      await reload();
    } catch (e) {
      error.textContent = String(e);
      busy(false, buttons);
    }
  };

  choose.addEventListener("click", () => void run(() => settingsApi.chooseProfileSyncFolder()));
  openDb.addEventListener("click", () => void run(() => settingsApi.openDatabaseFolder()));
  upload.addEventListener("click", () => void run(() => settingsApi.uploadProfileBackup(), "Backup uploaded."));
  restore.addEventListener("click", async () => {
    const confirmed = await confirmDialog(
      "This will replace the current local database with the selected backup. Xareon will first create a safety copy, then restart.",
      { confirmLabel: "Restore and restart", danger: true },
    );
    if (confirmed) void run(() => settingsApi.restoreProfileBackup());
  });

  const folder = info.folder ?? "No folder selected";
  const detail = info.statusDetail ? ` ${info.statusDetail}` : "";
  return el("section", { class: "sync-settings" }, [
    el("div", { class: "sync-heading" }, [
      el("div", {}, [
        el("h2", {}, ["Profile backup"]),
        el("p", { class: "field-hint" }, [
          "Manual synchronization through a local Google Drive folder on this device.",
        ]),
      ]),
      choose,
    ]),
    el("div", { class: "sync-folder", title: folder }, [folder]),
    el("div", { class: `sync-status sync-status-${info.status}` }, [
      statusLabel(info.status),
      detail,
    ]),
    el("div", { class: "sync-metadata" }, [
      marker("Last upload on this device", info.lastUploadAt),
      marker("Last restore on this device", info.lastRestoreAt),
      marker("Cloud backup created", info.backupCreatedAt, info.backupPlatform),
    ]),
    el("div", { class: "sync-actions" }, [openDb, upload, restore]),
  ]);
}

function marker(label: string, timestamp: number | null, suffix: string | null = null): HTMLElement {
  const value = timestamp === null ? "Never" : new Date(timestamp * 1000).toLocaleString();
  return el("div", {}, [
    el("span", {}, [label]),
    el("strong", {}, [suffix ? `${value} · ${suffix}` : value]),
  ]);
}

function statusLabel(status: ProfileSyncStatus): string {
  const labels: Record<ProfileSyncStatus, string> = {
    folderNotSelected: "Choose a Google Drive folder to enable backups.",
    backupUnavailable: "No backup is available in the selected folder.",
    upToDate: "Local database and backup are up to date.",
    localNewer: "Local database is newer.",
    backupNewer: "Backup is newer.",
    conflict: "Local database and backup have diverged.",
    invalidBackup: "The backup is invalid.",
  };
  return labels[status];
}

function shortcutField(initial: string | null): {
  row: HTMLElement;
  value: () => string | null;
  set: (value: string | null) => void;
} {
  let shortcut = initial;
  let suspended = false;
  const input = el("input", {
    type: "text",
    readOnly: true,
    class: "shortcut-input",
    placeholder: "Click and press a shortcut",
  });

  const render = (): void => {
    input.value = shortcut ? displayShortcut(shortcut) : "";
  };
  const resume = async (): Promise<void> => {
    if (!suspended) return;
    suspended = false;
    await settingsApi.resumePlayTrackingShortcut();
  };

  input.addEventListener("focus", async () => {
    input.value = "Press shortcut…";
    try {
      await settingsApi.suspendPlayTrackingShortcut();
      suspended = true;
    } catch {
      render();
    }
  });
  input.addEventListener("blur", () => {
    render();
    void resume();
  });
  input.addEventListener("keydown", (event) => {
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      input.blur();
      return;
    }
    if (event.key === "Backspace" || event.key === "Delete") {
      shortcut = null;
      render();
      input.blur();
      return;
    }
    const captured = captureShortcut(event);
    if (!captured) return;
    shortcut = captured;
    render();
    input.blur();
  });
  render();

  return {
    row: el("label", { class: "field field-wide" }, [
      el("span", {}, ["Play/Stop global shortcut"]),
      input,
      el("span", { class: "field-hint" }, [
        "Works while Xareon is in the background. Stops the active session, or starts the most recently played game. Press Backspace to disable.",
      ]),
    ]),
    value: () => shortcut,
    set: (value) => {
      shortcut = value;
      render();
    },
  };
}

function captureShortcut(event: KeyboardEvent): string | null {
  if (["Meta", "Control", "Alt", "Shift"].includes(event.key)) return null;
  if (!event.metaKey && !event.ctrlKey && !event.altKey) return null;

  const modifiers: string[] = [];
  if (event.metaKey) modifiers.push("Command");
  if (event.ctrlKey) modifiers.push("Control");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");

  const named: Record<string, string> = {
    " ": "Space",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Enter: "Enter",
    Tab: "Tab",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
  };
  const key = named[event.key] ?? (event.key.length === 1 ? event.key.toUpperCase() : event.key);
  return [...modifiers, key].join("+");
}

function displayShortcut(value: string): string {
  return value
    .replace("CmdOrCtrl", navigator.platform.includes("Mac") ? "⌘" : "Ctrl")
    .replace("Command", "⌘")
    .replace("Control", "Ctrl")
    .replace("Alt", navigator.platform.includes("Mac") ? "⌥" : "Alt")
    .replace("Shift", "⇧")
    .split("+")
    .join(" + ");
}
