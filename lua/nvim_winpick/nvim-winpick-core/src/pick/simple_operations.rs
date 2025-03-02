use super::Opts;

pub(crate) fn pick_focus_window(opts: &Opts) -> anyhow::Result<()> {
    let Some(window) = super::pick_window(opts)? else {
        return Ok(());
    };
    nvim_oxi::api::set_current_win(&window)?;
    Ok(())
}

pub(crate) fn pick_close_window(opts: &Opts) -> anyhow::Result<()> {
    for window in super::try_pick_multi_window(opts)? {
        window.close(false)?;
    }
    Ok(())
}

pub(crate) fn pick_swap_window(focus_new: bool, opts: &Opts) -> anyhow::Result<()> {
    // Race condition here, buffer on window changes between checks
    let mut cur_win = nvim_oxi::api::get_current_win();
    let cur_buf = cur_win.get_buf()?;
    let Some(mut target_win) = super::pick_window(opts)? else {
        return Ok(());
    };
    let target_buf = target_win.get_buf()?;
    cur_win.set_buf(&target_buf)?;
    target_win.set_buf(&cur_buf)?;
    if focus_new {
        nvim_oxi::api::set_current_win(&target_win)?;
    }

    Ok(())
}
