//! Navigation logic for seamless Neovim/tmux pane traversal.
//!
//! Detects whether the current Neovim window is at the edge of its split
//! layout. When at the edge, navigation is delegated to tmux via
//! `tmux select-pane`.

use std::env;
use std::process::Command;

use nvim_oxi::api;

/// Cardinal direction for split navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

impl Direction {
    /// The Vim `wincmd` character for this direction.
    const fn wincmd_char(self) -> char {
        match self {
            Self::Left => 'h',
            Self::Down => 'j',
            Self::Up => 'k',
            Self::Right => 'l',
        }
    }

    /// The tmux `select-pane` flag for this direction.
    const fn tmux_flag(self) -> &'static str {
        match self {
            Self::Left => "-L",
            Self::Down => "-D",
            Self::Up => "-U",
            Self::Right => "-R",
        }
    }
}

/// Returns `true` if we are running inside a tmux session.
fn in_tmux() -> bool {
    env::var_os("TMUX").is_some_and(|v| !v.is_empty())
}

/// Attempt to navigate in `dir`. If at the edge of all Neovim splits,
/// delegate to tmux (when available).
pub fn navigate(dir: Direction) {
    // Snapshot the current window number before attempting the move.
    let win_before: i64 = api::eval("winnr()").unwrap_or(0);

    // Try to move within Neovim.
    let cmd = format!("wincmd {}", dir.wincmd_char());
    if api::command(&cmd).is_err() {
        return;
    }

    let win_after: i64 = api::eval("winnr()").unwrap_or(0);

    // If the window number didn't change, we're at the edge.
    if win_before == win_after && in_tmux() {
        tmux_select_pane(dir);
    }
}

/// Send a `tmux select-pane` command for the given direction.
fn tmux_select_pane(dir: Direction) {
    let _ = Command::new("tmux")
        .args(["select-pane", dir.tmux_flag()])
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_wincmd_chars() {
        assert_eq!(Direction::Left.wincmd_char(), 'h');
        assert_eq!(Direction::Down.wincmd_char(), 'j');
        assert_eq!(Direction::Up.wincmd_char(), 'k');
        assert_eq!(Direction::Right.wincmd_char(), 'l');
    }

    #[test]
    fn direction_tmux_flags() {
        assert_eq!(Direction::Left.tmux_flag(), "-L");
        assert_eq!(Direction::Down.tmux_flag(), "-D");
        assert_eq!(Direction::Up.tmux_flag(), "-U");
        assert_eq!(Direction::Right.tmux_flag(), "-R");
    }

    #[test]
    fn in_tmux_detects_env() {
        // SAFETY: This test manipulates environment variables. It must not
        // run in parallel with other tests that read `TMUX`.
        unsafe {
            // Without TMUX set, should return false.
            env::remove_var("TMUX");
            assert!(!in_tmux());

            // With TMUX set to a non-empty value, should return true.
            env::set_var("TMUX", "/tmp/tmux-1000/default,12345,0");
            assert!(in_tmux());

            // With TMUX set to empty, should return false.
            env::set_var("TMUX", "");
            assert!(!in_tmux());

            // Clean up.
            env::remove_var("TMUX");
        }
    }
}
