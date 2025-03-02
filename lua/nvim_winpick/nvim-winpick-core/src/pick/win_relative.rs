use anyhow::Context;

use crate::{draw::FloatingLetterDrawer, win::open_split_with};

use super::Opts;

pub(crate) fn pick_win_relative(
    path: &str,
    focus_new: bool,
    relative_chars: &str,
    opts: &Opts,
) -> anyhow::Result<()> {
    let refocus = (!focus_new).then(nvim_oxi::api::get_current_win);
    let Some(mut win) = crate::pick::pick_window(opts)? else {
        return Ok(());
    };
    nvim_oxi::api::set_current_win(&win).context("failed to set focus window to picked window")?;
    let mut drawer = FloatingLetterDrawer::new_draw_within(relative_chars, &win);
    drawer.draw_multi()?;
    nvim_oxi::api::command("redraw").context("failed to redraw")?;
    let ch: u32 = nvim_oxi::api::call_function("getchar", ((),)).context("failed to get char")?;
    let ch = char::from_u32(ch).with_context(|| format!("invalid char picked: {ch}"))?;
    nvim_oxi::api::command("redraw").context("failed to redraw")?;
    let pos = drawer.pos_for_char(ch);
    let Some(pos) = pos else {
        return Ok(());
    };
    drop(drawer);
    open_split_with(path, refocus.as_ref(), &mut win, pos)?;

    Ok(())
}
