import { settingsApi } from "../api/settings";
import { clear, el } from "../ui/dom";
import type { Settings } from "../types/settings";

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
      const settings = await settingsApi.get();
      clear(container);
      container.append(form(settings));
    } catch (e) {
      clear(container);
      container.append(el("p", { class: "form-error" }, [`Failed to load: ${String(e)}`]));
    }
  };

  void load();
}

function form(settings: Settings): HTMLElement {
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
    "Your public, human-readable handle in Xareon (e.g. vitalii). Shared with friends and used as your Google Drive folder name. Not a UUID.",
  );
  const googleDriveFolder = field(
    "googleDriveFolder",
    "Google Drive folder URL",
    settings.googleDriveFolder,
    "Link to your shared Google Drive folder, used later for synchronization.",
  );

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
          googleDriveFolder: text(googleDriveFolder.input),
        };
        try {
          const saved = await settingsApi.update(input);
          userIdentifier.input.value = saved.userIdentifier ?? "";
          googleDriveFolder.input.value = saved.googleDriveFolder ?? "";
          message.textContent = "Settings saved.";
        } catch (e) {
          error.textContent = String(e);
        }
      },
    },
    [
      el("div", { class: "view-header" }, [el("h1", {}, ["Settings"])]),
      el("div", { class: "form-grid" }, [userIdentifier.row, googleDriveFolder.row]),
      error,
      message,
      el("div", { class: "modal-actions" }, [
        el("button", { type: "submit", class: "btn btn-primary" }, ["Save"]),
      ]),
    ],
  );
}
