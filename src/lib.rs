//! Watari (渡り) — seamless window navigation between Neovim and tmux panes.
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.
//!
//! # Behavior
//!
//! Registers `<C-h>`, `<C-j>`, `<C-k>`, `<C-l>` in normal mode. Each binding
//! first tries to move to an adjacent Neovim split in the corresponding
//! direction. When the cursor is already at the outermost split edge **and**
//! `$TMUX` is set, the key press is forwarded to `tmux select-pane` so
//! navigation crosses the tmux/Neovim boundary seamlessly.

mod nav;

use nav::Direction;
use nvim_oxi as oxi;
use nvim_oxi::api;
use nvim_oxi::api::opts::SetKeymapOpts;
use nvim_oxi::api::types::Mode;

/// Register a navigation keymap for one direction.
fn register_nav_keymap(lhs: &str, dir: Direction, desc: &str) -> oxi::Result<()> {
    let opts = SetKeymapOpts::builder()
        .silent(true)
        .noremap(true)
        .desc(desc)
        .callback(move |()| {
            nav::navigate(dir);
            Ok::<(), oxi::Error>(())
        })
        .build();

    api::set_keymap(Mode::Normal, lhs, "", &opts)?;
    Ok(())
}

#[oxi::plugin]
fn watari() -> oxi::Result<()> {
    register_nav_keymap("<C-h>", Direction::Left, "Watari: navigate left")?;
    register_nav_keymap("<C-j>", Direction::Down, "Watari: navigate down")?;
    register_nav_keymap("<C-k>", Direction::Up, "Watari: navigate up")?;
    register_nav_keymap("<C-l>", Direction::Right, "Watari: navigate right")?;
    Ok(())
}
