use anyhow::Context;
use nvim_oxi::api::Window;

use crate::opts::FilterRules;

impl FilterRules {
    pub(crate) fn filter(&self, target_win: &Window, current_win: &Window) -> anyhow::Result<bool> {
        if !self.include_current_win && current_win == target_win {
            return Ok(false);
        }
        let Ok(cfg) = target_win.get_config() else {
            return Ok(false);
        };
        if !self.include_unfocusable_windows && cfg.focusable == Some(false) {
            return Ok(false);
        }
        if !self.include_floating && cfg.relative.is_some() {
            return Ok(false);
        }
        if !self.file_path_contains.is_empty()
            || !self.file_name_contains.is_empty()
            || !self.bo.filetype.is_empty()
            || !self.bo.buftype.is_empty()
        {
            let buf = target_win.get_buf().context("failed to get window buf")?;
            // Alternative is umapped in nvim_oxi so far
            #[allow(deprecated)]
            if !self.bo.filetype.is_empty() {
                let ft: Option<String> = buf
                    .get_option("filetype")
                    .context("failed to get buf filetype")?;
                if let Some(ft) = ft {
                    if self.bo.filetype.iter().any(|filter_ft| filter_ft == &ft) {
                        return Ok(false);
                    }
                }
            }
            // Alternative is unmapped in nvim_oxi so far
            #[allow(deprecated)]
            if !self.bo.buftype.is_empty() {
                let bt: Option<String> = buf
                    .get_option("buftype")
                    .context("failed to get buf buftype")?;
                if let Some(ft) = bt {
                    if self.bo.buftype.iter().any(|filter_ft| filter_ft == &ft) {
                        return Ok(false);
                    }
                }
            }
            if !self.file_name_contains.is_empty() || !self.file_path_contains.is_empty() {
                let buf_file = buf.get_name().context("failed to get buf file")?;
                let path_utf8 = buf_file.display().to_string();
                if self
                    .file_path_contains
                    .iter()
                    .any(|fp| path_utf8.contains(fp))
                {
                    return Ok(false);
                }
                if !self.file_name_contains.is_empty() {
                    if let Some(file_name) = buf_file.file_name() {
                        if let Some(utf8) = file_name.to_str() {
                            if self.file_name_contains.iter().any(|f| utf8.contains(f)) {
                                return Ok(false);
                            }
                        }
                    }
                }
            }
        }
        Ok(true)
    }
}
