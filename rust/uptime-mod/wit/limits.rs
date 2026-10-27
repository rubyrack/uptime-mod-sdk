// The caps the host holds a step's output to. The game's mod host and the Rust
// SDK both read this file. The Lua guest's `lua/guest/src/uptime_limits.h`
// copies the values, and a host test checks that the copy matches.

/// Bytes of commands, panels and toasts in one step output, panel ids included.
pub const OUTPUT_BYTES: usize = 256 << 10;

/// Bytes of one encoded command.
pub const COMMAND_BYTES: usize = 16 << 10;

/// Bytes of mod state the host keeps.
pub const STATE_BYTES: usize = 1 << 20;

/// Bytes of one panel's encoded update.
pub const PANEL_BYTES: usize = 16 << 10;

/// Toasts one step may return. A step returning more has all of them refused.
pub const TOASTS_PER_STEP: usize = 2;

/// Bytes of one toast's text. An empty text is refused too.
pub const TOAST_TEXT_BYTES: usize = 160;
