use crate::{
    ctx::Context,
    draw::{FloatingBigLetterDrawer, FloatingLetterDrawer, PickBetweenWindowsDrawer},
    notify::notify_warn,
    opts::Opts,
    win::get_windows,
};
use anyhow::{Context as _, Result};
use nvim_oxi::api::{opts::SetHighlightOptsBuilder, Window};

pub(crate) mod simple_open;
pub(crate) mod simple_operations;
pub(crate) mod win_relative;

pub(crate) fn pick_window(opts: &Opts, ctx: &mut Context) -> Result<Option<Window>> {
    let windows = get_windows(|_| true)?;
    if windows.is_empty() {
        return Ok(None);
    }
    let mut filtered_windows = Vec::with_capacity(windows.len());
    let current_win = ctx.get_current_win();
    for win in windows {
        if opts.filter_rules.filter(&win, &current_win)? {
            filtered_windows.push(win);
        }
    }
    if filtered_windows.is_empty() {
        notify_warn("No windows left after filtering");
        return Ok(None);
    }
    if filtered_windows.len() == 1 && opts.filter_rules.autoselect_one {
        return Ok(filtered_windows.pop());
    }
    let win = match opts.hint {
        crate::hint::Hint::FloatingBigLetter => exec_draw(FloatingBigLetterDrawer::new(
            &opts.selection_chars,
            &filtered_windows,
        )),
        crate::hint::Hint::FloatingLetter => exec_draw(FloatingLetterDrawer::new_pick_between(
            &opts.selection_chars,
            &filtered_windows,
        )),
    }?;
    Ok(win)
}

fn exec_draw<D>(mut drawer: D) -> anyhow::Result<Option<Window>>
where
    D: PickBetweenWindowsDrawer,
{
    drawer.draw()?;
    nvim_oxi::api::command("redraw").context("failed to redraw")?;
    let ch: u32 = nvim_oxi::api::call_function("getchar", ((),)).context("failed to get char")?;
    let ch = char::from_u32(ch).with_context(|| format!("invalid char picked: {ch}"))?;
    nvim_oxi::api::command("redraw").context("failed to redraw")?;
    let win = drawer.target_win_for_char(ch);
    Ok(win)
}

pub(crate) fn try_pick_multi_window(opts: &Opts, ctx: &mut Context) -> Result<Vec<Window>> {
    let windows = get_windows(|_| true)?;
    if windows.is_empty() {
        return Ok(vec![]);
    }
    let mut filtered_windows = Vec::with_capacity(windows.len());
    let current_win = ctx.get_current_win();
    for win in windows {
        if opts.filter_rules.filter(&win, &current_win)? {
            filtered_windows.push(win);
        }
    }
    if filtered_windows.is_empty() {
        notify_warn("No windows left after filtering");
        return Ok(vec![]);
    }
    if filtered_windows.len() == 1 && opts.filter_rules.autoselect_one {
        return Ok(filtered_windows);
    }
    let win = if let Some(multiselect) = opts.multiselect {
        match opts.hint {
            crate::hint::Hint::FloatingBigLetter => exec_multi_draw(
                FloatingBigLetterDrawer::new(&opts.selection_chars, &filtered_windows),
                multiselect.trigger_char,
                multiselect.commit_char,
            ),
            crate::hint::Hint::FloatingLetter => exec_multi_draw(
                FloatingLetterDrawer::new_pick_between(&opts.selection_chars, &filtered_windows),
                multiselect.trigger_char,
                multiselect.commit_char,
            ),
        }?
    } else {
        let win = match opts.hint {
            crate::hint::Hint::FloatingBigLetter => exec_draw(FloatingBigLetterDrawer::new(
                &opts.selection_chars,
                &filtered_windows,
            )),
            crate::hint::Hint::FloatingLetter => exec_draw(FloatingLetterDrawer::new_pick_between(
                &opts.selection_chars,
                &filtered_windows,
            )),
        }?;
        let mut v = vec![];
        if let Some(win) = win {
            v.push(win);
        }
        v
    };
    Ok(win)
}

fn exec_multi_draw<D>(
    mut drawer: D,
    multi_select_char: char,
    commit_char: char,
) -> anyhow::Result<Vec<Window>>
where
    D: PickBetweenWindowsDrawer,
{
    drawer.draw()?;
    nvim_oxi::api::command("redraw").context("failed to redraw")?;
    let ch: u32 = nvim_oxi::api::call_function("getchar", ((),)).context("failed to get char")?;
    let ch = char::from_u32(ch).with_context(|| format!("invalid char picked: {ch}"))?;
    // It doesn't make sense to use a hashset for such a limited collection, likely slower, and
    // more inconvenient because it'll need at least one realloc before returning it.
    let mut wins = vec![];
    let ns = nvim_oxi::api::create_namespace("nvim_winpick");
    let opts = SetHighlightOptsBuilder::default()
        .foreground("#f09ea1")
        .build();
    nvim_oxi::api::set_hl(ns, "Normal", &opts)?;
    if ch == multi_select_char {
        loop {
            let ch: u32 =
                nvim_oxi::api::call_function("getchar", ((),)).context("failed to get char")?;
            let ch = char::from_u32(ch).with_context(|| format!("invalid char picked: {ch}"))?;
            if ch == commit_char {
                break;
            }
            let target_and_drawn = drawer.target_and_drawn_win_for_char(ch);
            let Some((tgt_win, mut drawn_win)) = target_and_drawn else {
                return Ok(vec![]);
            };

            let mut was_present = false;
            for i in 0..wins.len() {
                if wins[i] == tgt_win {
                    // Not preserving any order here, could implement but would be slower
                    wins.swap_remove(i);
                    drawn_win.set_hl(0)?;
                    was_present = true;
                    break;
                }
            }
            if !was_present {
                wins.push(tgt_win);
                drawn_win.set_hl(ns)?;
            }
            nvim_oxi::api::command("redraw").context("failed to redraw")?;
        }
    } else if let Some(win) = drawer.target_win_for_char(ch) {
        wins.push(win);
    }

    Ok(wins)
}
