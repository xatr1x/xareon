/** Mirror of the Rust journal domain types (camelCase on the wire). */

export interface JournalEntry {
  id: number;
  gameId: number;
  body: string;
  createdAt: string;
  updatedAt: string;
}

export interface NewJournalEntry {
  gameId: number;
  body: string;
}

export interface JournalEntryUpdate {
  body: string;
}
