use std::{f64, ops::Sub};

use anyhow::{bail, Context, Result};
use nvim_oxi::api::types::WindowConfig;

#[derive(Clone, Copy)]
pub(crate) struct WindowDimensions {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl WindowDimensions {
    pub(crate) fn try_from_win_cfg(win_cfg: &WindowConfig) -> Result<Self> {
        let width = win_cfg.width.context("failed to get window width")?;
        let height = win_cfg.height.context("failed to get window height")?;
        Ok(Self { width, height })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Positions {
    TopFullHor,
    RightFullVer,
    BotFullHor,
    LeftFullVer,
    SplitTop,
    SplitRight,
    SplitBot,
    SplitLeft,
    Center,
}

impl Positions {
    const NUM_POSITIONS: usize = 9;
    pub(crate) fn iter() -> [Self; Self::NUM_POSITIONS] {
        [
            Self::TopFullHor,
            Self::RightFullVer,
            Self::BotFullHor,
            Self::LeftFullVer,
            Self::SplitTop,
            Self::SplitRight,
            Self::SplitBot,
            Self::SplitLeft,
            Self::Center,
        ]
    }

    pub(crate) fn calculate_indicator_positions(
        win: WindowDimensions,
        overlay: WindowDimensions,
    ) -> anyhow::Result<[ScreenPoint; Self::NUM_POSITIONS]> {
        let win_center_x: f64 = f64::from(win.width) / 2.0;
        let overlay_center_x: f64 = f64::from(overlay.width) / 2.0;
        let center_left_x = win_center_x.sub(overlay_center_x);
        if center_left_x < 0.0 {
            bail!("not enough space to draw characters");
        }
        let win_height = f64::from(win.height);
        let win_width = f64::from(win.width);
        let overlay_height = f64::from(overlay.height);
        let top_full_hor = ScreenPoint {
            x: center_left_x,
            // I don't know why this is correct but it is, half an overlay's worth of padding from top
            // edge
            y: 0.0,
        };
        let bot_full_hor = ScreenPoint {
            x: center_left_x,
            // Don't know why this is correct either, but it is, half an overlay's worth of padding
            // from the bottom edge
            y: win_height - 3.0 * overlay_height,
        };
        let win_center_y = f64::from(win.height) / 2.0;
        let overlay_center_y = f64::from(overlay.height) / 2.0;
        let center_overlay_y = win_center_y - overlay_height;
        let center = ScreenPoint {
            x: center_left_x,
            y: center_overlay_y,
        };
        let win_vert_split_length = f64::from(win.height) / 4.0;
        let win_split_top_center_y = win_vert_split_length - overlay_center_y;
        let win_split_bot_center_y = 3.0 * (win_vert_split_length - overlay_center_y);
        let split_top = ScreenPoint {
            x: center_left_x,
            y: win_split_top_center_y,
        };
        let split_bot = ScreenPoint {
            x: center_left_x,
            y: win_split_bot_center_y,
        };

        let win_hor_split_length = f64::from(win.width) / 4.0;
        let win_split_left_center_x = win_hor_split_length - overlay_center_x;
        let win_split_right_center_x = 3.0 * win_hor_split_length - overlay_center_x;
        let split_right = ScreenPoint {
            x: win_split_right_center_x,
            y: center_overlay_y,
        };
        let split_left = ScreenPoint {
            x: win_split_left_center_x,
            y: center_overlay_y,
        };
        let left_full_ver = ScreenPoint {
            x: 0.0,
            y: center_overlay_y,
        };
        let right_full_ver = ScreenPoint {
            x: win_width - 4.0 * overlay_center_x,
            y: center_overlay_y,
        };

        Ok([
            top_full_hor,
            right_full_ver,
            bot_full_hor,
            left_full_ver,
            split_top,
            split_right,
            split_bot,
            split_left,
            center,
        ])
    }
}

pub(crate) struct ScreenPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

pub(crate) fn get_overlay_centered_position(
    win_cfg: &WindowConfig,
    overlay_width: u32,
    overlay_height: u32,
) -> Result<ScreenPoint> {
    let width = win_cfg.width.context("failed to get window width")?;
    let height = win_cfg.height.context("failed to get window height")?;
    Ok(ScreenPoint {
        x: width
            .saturating_sub(overlay_width)
            .checked_div(2)
            .unwrap_or_default()
            .into(),
        y: height
            .saturating_sub(overlay_height)
            .checked_div(2)
            .unwrap_or_default()
            .into(),
    })
}
