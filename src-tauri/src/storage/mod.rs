//! File storage layer — covers, screenshots, backups and other on-disk assets.
//!
//! Reserved for upcoming file-backed features. Any feature that persists files
//! must go through this layer rather than calling filesystem APIs directly from
//! services or commands, so paths and lifecycle stay in one place. No
//! implementation yet (no file-backed features exist).
