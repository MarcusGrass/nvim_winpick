use nvim_oxi::api::Window;

/// Simple context where immutable things that may be requested a lot can be cached
pub(crate) struct Context {
    current_win: Option<Window>,
}

impl Context {
    pub(crate) const DEFAULT: Self = Self { current_win: None };
    pub(crate) fn get_current_win(&mut self) -> Window {
        if let Some(win) = self.current_win.clone() {
            return win;
        }
        let current_win = nvim_oxi::api::get_current_win();
        self.current_win = Some(current_win.clone());
        current_win
    }
}
