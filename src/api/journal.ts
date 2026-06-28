import { invoke } from "@tauri-apps/api/core";
import type { JournalEntry, JournalEntryUpdate, NewJournalEntry } from "../types/journal";

/** Thin typed wrapper around the Tauri `*_journal_entry` commands. */
export const journalApi = {
  listForGame(gameId: number): Promise<JournalEntry[]> {
    return invoke<JournalEntry[]>("list_journal_entries", { gameId });
  },
  create(input: NewJournalEntry): Promise<JournalEntry> {
    return invoke<JournalEntry>("create_journal_entry", { input });
  },
  update(id: number, update: JournalEntryUpdate): Promise<JournalEntry> {
    return invoke<JournalEntry>("update_journal_entry", { id, update });
  },
  delete(id: number): Promise<void> {
    return invoke<void>("delete_journal_entry", { id });
  },
};
