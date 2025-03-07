use nvim_oxi::{Dictionary, Function, Object};

#[nvim_oxi::plugin]
pub fn nvim_winpick() -> Dictionary {
    // lib-native parsing can panic, crashing neovim, do the conversion manually
    let setup = Function::from_fn(nvim_winpick_core::setup);
    let pick_window = Function::from_fn(nvim_winpick_core::pick_window);
    let pick_multiple_windows = Function::from_fn(nvim_winpick_core::pick_multiple_windows);
    let pick_focus_window = Function::from_fn(nvim_winpick_core::pick_focus_window);
    let pick_close_window = Function::from_fn(nvim_winpick_core::pick_close_window);
    let pick_swap_window = Function::from_fn(nvim_winpick_core::pick_swap_window);
    let open_split_window = Function::from_fn(nvim_winpick_core::open_split);
    let open_over_window = Function::from_fn(nvim_winpick_core::open_over);
    let pick_win_relative = Function::from_fn(nvim_winpick_core::pick_win_relative);
    let entries: [(&str, Object); 9] = [
        ("setup", setup.into()),
        ("pick_window", pick_window.into()),
        ("pick_multiple_windows", pick_multiple_windows.into()),
        ("pick_focus_window", pick_focus_window.into()),
        ("pick_close_window", pick_close_window.into()),
        ("pick_swap_window", pick_swap_window.into()),
        ("pick_open_split", open_split_window.into()),
        ("pick_open_over", open_over_window.into()),
        ("pick_win_relative", pick_win_relative.into()),
    ];
    Dictionary::from_iter(entries)
}
