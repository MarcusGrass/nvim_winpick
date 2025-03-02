use nvim_oxi::Dictionary;

pub(crate) fn notify_error(msg: &str) {
    let _ = nvim_oxi::api::notify(
        msg,
        nvim_oxi::api::types::LogLevel::Error,
        &Dictionary::default(),
    );
}

pub(crate) fn notify_warn(msg: &str) {
    let _ = nvim_oxi::api::notify(
        msg,
        nvim_oxi::api::types::LogLevel::Warn,
        &Dictionary::default(),
    );
}
