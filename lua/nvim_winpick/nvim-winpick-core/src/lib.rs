use notify::notify_error;
use nvim_oxi::{api::Window, Object};
mod buf;
mod chars;
mod draw;
mod filter;
mod geometry;
mod hint;
mod notify;
mod opts;
mod pick;
mod win;

pub use hint::Hint;
pub use opts::{OpenOverOpts, OpenRelativeOpts, OpenSplitOpts, Opts};

pub fn setup(opts: Option<Object>) {
    let Some(opts) = safe_parse_opts(opts) else {
        return;
    };
    if let Err(e) = Opts::setup_opts(opts) {
        notify_error(&format!("[nvim_winpick] failed to setup {e:#?}"));
    }
}

#[must_use]
pub fn pick_window(opts: Option<Object>) -> Option<Window> {
    let opts = safe_parse_opts(opts)?;
    match pick::pick_window(&opts) {
        Ok(v) => v,
        Err(e) => {
            notify_error(&format!("[nvim_winpick] failed to pick window {e:#?}"));
            None
        }
    }
}

#[must_use]
pub fn pick_multiple_windows(opts: Option<Object>) -> Vec<Window> {
    let Some(opts) = safe_parse_opts(opts) else {
        return vec![];
    };
    match pick::try_pick_multi_window(&opts) {
        Ok(w) => w,
        Err(e) => {
            notify_error(&format!(
                "[nvim_winpick] failed to pick multiple windows: {e:#?}"
            ));
            vec![]
        }
    }
}

/// Returning a result causes a panic that exits wim, very annoying, wrap and notify instead
#[must_use]
pub fn safe_parse_opts(opts: Option<Object>) -> Option<Opts> {
    let opts = if let Some(opts) = opts {
        return match opts::Opts::parse_obj(opts) {
            Ok(o) => Some(o),
            Err(e) => {
                notify_error(&format!("[nvim_winpic] invalid opts: {e:#?}"));
                None
            }
        };
    } else {
        opts::Opts::default()
    };
    Some(opts)
}

pub fn pick_focus_window(opts: Option<Object>) {
    let Some(opts) = safe_parse_opts(opts) else {
        return;
    };
    match pick::simple_operations::pick_focus_window(&opts) {
        Ok(()) => {}
        Err(e) => {
            notify_error(&format!(
                "[nvim_winpick] failed to pick focus window: {e:#?}"
            ));
        }
    }
}

pub fn pick_close_window(opts: Option<Object>) {
    let Some(opts) = safe_parse_opts(opts) else {
        return;
    };
    match pick::simple_operations::pick_close_window(&opts) {
        Ok(()) => {}
        Err(e) => {
            notify_error(&format!(
                "[nvim_winpick] failed to pick window to close: {e:#?}"
            ));
        }
    }
}

pub fn pick_swap_window(opts: Option<Object>) {
    let Some(opts) = safe_parse_opts(opts) else {
        return;
    };
    match pick::simple_operations::pick_swap_window(true, &opts) {
        Ok(()) => {}
        Err(e) => {
            notify_error(&format!(
                "[nvim_winpick] failed to pick window to swap with: {e:#?}"
            ));
        }
    }
}

pub fn open_split(opts: Option<Object>) {
    let Some(opts) = opts else {
        notify_error("[nvim_winpick] failed to open split, no opts supplied, needs at least { path = <path> }");
        return;
    };
    let opts = match OpenSplitOpts::parse_obj(opts) {
        Ok(opts) => opts,
        Err(e) => {
            notify_error(&format!(
                "[nvim_winpick] failed to parse 'open_split_opts': {e:#?}"
            ));
            return;
        }
    };
    if let Err(e) = pick::simple_open::open_simple_split_at_win(
        opts.focus_new,
        opts.vertical,
        &opts.path,
        &opts.opts,
    ) {
        notify_error(&format!("[nvim_winpick] failed to open split: {e:#?}"));
    }
}

pub fn open_over(opts: Option<Object>) {
    {
        let Some(opts) = opts else {
            notify_error("[nvim_winpick] failed to open over, no opts supplied, needs at least { path = <path> }");
            return;
        };
        let opts = match OpenOverOpts::parse_obj(opts) {
            Ok(opts) => opts,
            Err(e) => {
                notify_error(&format!(
                    "[nvim_winpick] failed to parse 'open_over_opts': {e:#?}"
                ));
                return;
            }
        };
        if let Err(e) = pick::simple_open::open_over_win(&opts.path, opts.focus_new, &opts.opts) {
            notify_error(&format!("[nvim_winpick] failed to open over: {e:#?}"));
        }
    }
}

pub fn pick_win_relative(opts: Option<Object>) {
    let Some(opts) = opts else {
        notify_error(
            "[nvim_winpick] failed to pick_win_relative, no opts supplied needs at least { path = <path> }"
        );
        return;
    };
    let opts = match OpenRelativeOpts::parse_obj(opts) {
        Ok(opts) => opts,
        Err(e) => {
            notify_error(&format!(
                "[nvim_winpick] failed to parse 'open_over_opts': {e:#?}"
            ));
            return;
        }
    };
    if let Err(e) = pick::win_relative::pick_win_relative(
        &opts.path,
        opts.focus_new,
        &opts.relative_chars,
        &opts.opts,
    ) {
        notify_error(&format!("[nvim_winpick] failed to open over: {e:#?}"));
    }
}
