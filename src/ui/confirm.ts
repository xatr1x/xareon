import { el } from "./dom";

/**
 * A themed confirmation dialog returning a Promise<boolean>. Used instead of the
 * native `window.confirm`, which is unreliable inside the Tauri webview.
 */
export function confirmDialog(
  message: string,
  options: { confirmLabel?: string; danger?: boolean } = {},
): Promise<boolean> {
  return new Promise((resolve) => {
    const overlay = el("div", { class: "modal-overlay" });
    const finish = (value: boolean): void => {
      overlay.remove();
      resolve(value);
    };

    const confirmBtn = el(
      "button",
      { class: `btn ${options.danger ? "btn-danger" : "btn-primary"}`, onclick: () => finish(true) },
      [options.confirmLabel ?? "Confirm"],
    );

    overlay.append(
      el("div", { class: "modal confirm" }, [
        el("p", { class: "confirm-message" }, [message]),
        el("div", { class: "modal-actions" }, [
          el("button", { class: "btn", onclick: () => finish(false) }, ["Cancel"]),
          confirmBtn,
        ]),
      ]),
    );

    overlay.addEventListener("click", (event) => {
      if (event.target === overlay) finish(false);
    });
    document.body.append(overlay);
    confirmBtn.focus();
  });
}
