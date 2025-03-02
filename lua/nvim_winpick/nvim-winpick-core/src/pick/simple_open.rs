use anyhow::{Context, Result};
use nvim_oxi::api::types::WindowConfigBuilder;

use crate::win::open_split_with;

use super::Opts;

pub(crate) fn open_simple_split_at_win(
    focus_new: bool,
    vertical: bool,
    path: &str,
    opts: &Opts,
) -> Result<()> {
    let bufnr = crate::buf::load_file_to_hidden_buffer(path)?;
    let Some(win) = crate::pick::pick_window(opts)? else {
        return Ok(());
    };

    let mut opts_builder = WindowConfigBuilder::default();
    let mut opts = opts_builder.vertical(vertical).build();
    // Todo:  This should be exposed through the builder, it's not only through relative afaik,
    // check with nvim_oxi
    opts.win = Some(win);
    nvim_oxi::api::open_win(&bufnr, focus_new, &opts).context("failed to open window")?;
    Ok(())
}

pub(crate) fn open_over_win(path: &str, focus_new: bool, opts: &Opts) -> Result<()> {
    let refocus = (!focus_new).then(nvim_oxi::api::get_current_win);
    let Some(mut win) = crate::pick::pick_window(opts)? else {
        return Ok(());
    };
    open_split_with(
        path,
        refocus.as_ref(),
        &mut win,
        crate::geometry::Positions::Center,
    )
}
