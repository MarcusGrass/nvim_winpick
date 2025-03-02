use anyhow::{Context, Result};
use nvim_oxi::api::{
    opts::BufDeleteOptsBuilder,
    types::{WindowBorder, WindowBorderChar, WindowConfig, WindowStyle},
    Window,
};

use crate::geometry::{get_overlay_centered_position, Positions, WindowDimensions};

pub(crate) struct Drawer<'a, S> {
    chars: &'a str,
    draw_setting: S,
    drawn_windows: Vec<Window>,
}

impl Drawer<'_, PickBetweenWindows<'_>> {
    pub(crate) fn target_win_for_char(&self, user: char) -> Option<Window> {
        for (ch, win) in self.chars.chars().zip(self.draw_setting.0.iter()) {
            if ch.eq_ignore_ascii_case(&user) {
                return Some(win.clone());
            }
        }
        None
    }
    pub(crate) fn target_and_drawn_win_for_char(&self, user: char) -> Option<(Window, Window)> {
        for ((ch, drawn), target) in self
            .chars
            .chars()
            .zip(&self.drawn_windows)
            .zip(self.draw_setting.0.iter())
        {
            if ch.eq_ignore_ascii_case(&user) {
                return Some((target.clone(), drawn.clone()));
            }
        }
        None
    }
}

impl<S> Drawer<'_, S> {
    pub(crate) fn clear(&mut self) -> Result<()> {
        for win in self.drawn_windows.drain(..) {
            if win.is_valid() {
                let buf = win
                    .get_buf()
                    .context("failed to clean window, failed to get buf")?;
                win.close(true).context("failed to close window")?;
                let mut bdel = BufDeleteOptsBuilder::default();
                bdel.force(true);
                buf.delete(&bdel.build())
                    .context("failed to delete buffer")?;
            }
        }
        Ok(())
    }
}

impl<S> Drop for Drawer<'_, S> {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}

pub(crate) struct FloatingBigLetterDrawer<'a, S> {
    inner: Drawer<'a, S>,
}

pub(crate) trait PickBetweenWindowsDrawer {
    fn draw(&mut self) -> anyhow::Result<()>;
    fn target_win_for_char(&self, user: char) -> Option<Window>;
    fn target_and_drawn_win_for_char(&self, user: char) -> Option<(Window, Window)>;
}

impl<'a> FloatingBigLetterDrawer<'a, PickBetweenWindows<'a>> {
    pub(crate) fn new(chars: &'a str, windows: &'a [Window]) -> Self {
        Self {
            inner: Drawer {
                chars,
                draw_setting: PickBetweenWindows(windows),
                drawn_windows: Vec::new(),
            },
        }
    }

    fn show_letter_in_window(&mut self, window: Window, ch: char) -> Result<()> {
        let win_config = window.get_config().context("failed to get window config")?;
        let pos = get_overlay_centered_position(&win_config, BIG_CHAR_WIDTH, BIG_CHAR_HEIGHT)?;
        let lines = crate::chars::char_to_lines(ch)?;
        let lines = add_char_margin(lines);
        let width: u32 = lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .context("found no max length on rendered char lines, this is a bug")?
            .try_into()
            .context("got a width larger than a u32")?;
        let height: u32 = lines
            .len()
            .try_into()
            .context("got a length larger than a u32")?;
        let mut buffer = nvim_oxi::api::create_buf(false, true)
            .context("failed to create char display buffer")?;
        let mut wc = WindowConfig::default();
        wc.relative = Some(nvim_oxi::api::types::WindowRelativeTo::Window(
            window.clone(),
        ));
        wc.win = Some(window);
        wc.focusable = Some(true);
        wc.width = Some(width);
        wc.height = Some(height);
        wc.row = Some(pos.y);
        wc.col = Some(pos.x);
        wc.style = Some(WindowStyle::Minimal);
        wc.border = Some(border());
        let rendered = nvim_oxi::api::open_win(&buffer, false, &wc)
            .context("failed to open char display window")?;
        buffer
            .set_lines(.., false, lines)
            .context("failed to write char to display buffer")?;
        self.inner.drawn_windows.push(rendered);
        Ok(())
    }
}

impl<'a> PickBetweenWindowsDrawer for FloatingBigLetterDrawer<'a, PickBetweenWindows<'a>> {
    fn draw(&mut self) -> Result<()> {
        for (win, ch) in self
            .inner
            .draw_setting
            .0
            .iter()
            .zip(self.inner.chars.chars())
        {
            self.show_letter_in_window(win.clone(), ch)?;
        }
        Ok(())
    }

    #[inline]
    fn target_win_for_char(&self, user: char) -> Option<Window> {
        self.inner.target_win_for_char(user)
    }

    #[inline]
    fn target_and_drawn_win_for_char(&self, user: char) -> Option<(Window, Window)> {
        self.inner.target_and_drawn_win_for_char(user)
    }
}

pub(crate) struct PickBetweenWindowSplits<'a>(&'a Window);

pub(crate) struct PickBetweenWindows<'a>(&'a [Window]);

pub(crate) struct FloatingLetterDrawer<'a, S> {
    inner: Drawer<'a, S>,
}

impl<'a> FloatingLetterDrawer<'a, PickBetweenWindows<'a>> {
    pub(crate) fn new_pick_between(chars: &'a str, windows: &'a [Window]) -> Self {
        Self {
            inner: Drawer {
                chars,
                draw_setting: PickBetweenWindows(windows),
                drawn_windows: Vec::new(),
            },
        }
    }
    fn show_letter_in_window(&mut self, window: Window, ch: char) -> Result<()> {
        let win_config = window.get_config().context("failed to get window config")?;
        let pos = get_overlay_centered_position(&win_config, BIG_CHAR_WIDTH, BIG_CHAR_HEIGHT)?;
        let overlay_dims = WindowDimensions {
            width: 3,
            height: 1,
        };
        let mut buffer = nvim_oxi::api::create_buf(false, true)
            .context("failed to create char display buffer")?;
        let mut wc = WindowConfig::default();
        wc.relative = Some(nvim_oxi::api::types::WindowRelativeTo::Window(
            window.clone(),
        ));
        wc.win = Some(window);
        wc.focusable = Some(true);
        wc.width = Some(overlay_dims.width);
        wc.height = Some(overlay_dims.height);
        wc.row = Some(pos.y);
        wc.col = Some(pos.x);
        wc.style = Some(WindowStyle::Minimal);
        wc.border = Some(border());
        let rendered = nvim_oxi::api::open_win(&buffer, false, &wc)
            .context("failed to open char display window")?;
        buffer
            .set_lines(.., false, [format!(" {ch} ")])
            .context("failed to write char to display buffer")?;
        self.inner.drawn_windows.push(rendered);
        Ok(())
    }
}

impl<'a> PickBetweenWindowsDrawer for FloatingLetterDrawer<'a, PickBetweenWindows<'a>> {
    fn draw(&mut self) -> Result<()> {
        for (win, ch) in self
            .inner
            .draw_setting
            .0
            .iter()
            .zip(self.inner.chars.chars())
        {
            self.show_letter_in_window(win.clone(), ch)?;
        }
        Ok(())
    }

    #[inline]
    fn target_win_for_char(&self, user: char) -> Option<Window> {
        self.inner.target_win_for_char(user)
    }
    #[inline]
    fn target_and_drawn_win_for_char(&self, user: char) -> Option<(Window, Window)> {
        self.inner.target_and_drawn_win_for_char(user)
    }
}

impl<'a> FloatingLetterDrawer<'a, PickBetweenWindowSplits<'a>> {
    pub(crate) fn new_draw_within(chars: &'a str, window: &'a Window) -> Self {
        Self {
            inner: Drawer {
                chars,
                draw_setting: PickBetweenWindowSplits(window),
                drawn_windows: Vec::new(),
            },
        }
    }
    fn show_multi_letter_in_window(&mut self, window: &Window) -> Result<()> {
        let win_config = window.get_config().context("failed to get window config")?;
        let win_dims = WindowDimensions::try_from_win_cfg(&win_config)?;
        let overlay_dims = WindowDimensions {
            width: 3,
            height: 1,
        };
        let positions =
            crate::geometry::Positions::calculate_indicator_positions(win_dims, overlay_dims)?;
        // Ordering of this array is implicitly important, really dumb implementation by me
        let char_iter = self.inner.chars.chars().zip(positions);
        for (ch, pos) in char_iter {
            let line = format!(" {ch} ");
            let width: u32 = 3;
            let height: u32 = 1;
            let mut buffer = nvim_oxi::api::create_buf(false, true)
                .context("failed to create char display buffer")?;
            let mut wc = WindowConfig::default();
            wc.relative = Some(nvim_oxi::api::types::WindowRelativeTo::Window(
                window.clone(),
            ));
            wc.win = Some(window.clone());
            wc.focusable = Some(true);
            wc.width = Some(width);
            wc.height = Some(height);
            wc.row = Some(pos.y);
            wc.col = Some(pos.x);
            wc.style = Some(WindowStyle::Minimal);
            wc.border = Some(border());
            let rendered = nvim_oxi::api::open_win(&buffer, false, &wc)
                .context("failed to open char display window")?;
            self.inner.drawn_windows.push(rendered);
            buffer
                .set_lines(.., false, [line])
                .context("failed to write char to display buffer")?;
        }

        Ok(())
    }

    pub(crate) fn draw_multi(&mut self) -> Result<()> {
        self.show_multi_letter_in_window(self.inner.draw_setting.0)?;
        Ok(())
    }

    pub(crate) fn pos_for_char(&self, user: char) -> Option<Positions> {
        for (ch, pos) in self.inner.chars.chars().zip(Positions::iter()) {
            if ch.eq_ignore_ascii_case(&user) {
                return Some(pos);
            }
        }
        None
    }
}

pub(crate) const BIG_CHAR_WIDTH: u32 = 18;
pub(crate) const BIG_CHAR_HEIGHT: u32 = 8;

const BORDER_HL_GROUP: &str = "FloatBoarder";

fn border() -> WindowBorder {
    WindowBorder::Anal(
        WindowBorderChar::CharAndHlGroup(Some('╭'), BORDER_HL_GROUP.to_string()),
        WindowBorderChar::CharAndHlGroup(Some('─'), BORDER_HL_GROUP.to_string()),
        WindowBorderChar::CharAndHlGroup(Some('╮'), BORDER_HL_GROUP.to_string()),
        WindowBorderChar::CharAndHlGroup(Some('│'), BORDER_HL_GROUP.to_string()),
        WindowBorderChar::CharAndHlGroup(Some('╯'), BORDER_HL_GROUP.to_string()),
        WindowBorderChar::CharAndHlGroup(Some('─'), BORDER_HL_GROUP.to_string()),
        WindowBorderChar::CharAndHlGroup(Some('╰'), BORDER_HL_GROUP.to_string()),
        WindowBorderChar::CharAndHlGroup(Some('│'), BORDER_HL_GROUP.to_string()),
    )
}

fn add_char_margin(lines: &[&str]) -> Vec<String> {
    let mut max_text_width = 0;
    let mut centered_lines = Vec::new();
    for line in lines {
        let len = line.chars().count();
        if max_text_width < len {
            max_text_width = len;
        }
    }
    centered_lines.push(String::new());
    for line in lines {
        centered_lines.push(format!(" {line} "));
    }
    centered_lines.push(String::new());
    centered_lines
}
