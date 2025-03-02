use anyhow::{Context, Result};
use nvim_oxi::api::Window;

use crate::geometry::Positions;

pub(crate) fn get_windows<F: FnMut(&Window) -> bool>(filter_fn: F) -> Result<Vec<Window>> {
    let tab = nvim_oxi::api::get_current_tabpage();
    let windows = tab.list_wins()?.filter(filter_fn).collect();
    Ok(windows)
}

pub(crate) fn open_split_with(
    path: &str,
    keep_focus_at: Option<&Window>,
    window: &mut Window,
    pos: Positions,
) -> Result<()> {
    let with_pos = match pos {
        Positions::TopFullHor => {
            format!("topleft split {path}")
        }
        Positions::LeftFullVer => {
            format!("topleft vertical split {path}")
        }
        Positions::BotFullHor => {
            format!("botright split {path}")
        }
        Positions::RightFullVer => {
            format!("botright vertical split {path}")
        }
        Positions::Center => {
            let buf = crate::buf::load_file_to_hidden_buffer(path)?;
            window.set_buf(&buf).context("failed to set buffer")?;
            if let Some(refocus) = keep_focus_at {
                nvim_oxi::api::set_current_win(refocus).context("failed to refocus old window")?;
            } else {
                nvim_oxi::api::set_current_win(window).context("failed to focus new window")?;
            }
            return Ok(());
        }
        Positions::SplitTop => {
            format!("leftabove split {path}")
        }
        Positions::SplitRight => {
            format!("belowright vertical split {path}")
        }
        Positions::SplitBot => {
            format!("belowright split {path}")
        }
        Positions::SplitLeft => {
            format!("leftabove vertical split {path}")
        }
    };
    nvim_oxi::api::command(&with_pos).context("failed to run split command")?;
    Ok(())
}
