use std::sync::OnceLock;

use anyhow::{bail, Context};
use nvim_oxi::conversion::FromObject;
use nvim_oxi::{Array, Dictionary, Object};

use crate::hint::Hint;

static SETUP_OPTS: OnceLock<Opts> = OnceLock::new();
#[derive(Debug, Clone)]
#[cfg_attr(feature = "test", derive(Eq, PartialEq))]
pub struct Opts {
    pub filter_rules: FilterRules,
    pub selection_chars: String,
    pub hint: Hint,
    pub multiselect: Option<Multiselect>,
}

impl Opts {
    pub(crate) fn setup_opts(opts: Self) -> anyhow::Result<()> {
        SETUP_OPTS
            .set(opts)
            .map_err(|_| anyhow::anyhow!("tried to setup twice"))
    }
    pub(crate) fn parse_obj(object: Object) -> anyhow::Result<Self> {
        let dict = obj_to_dict(object).context("not a valid table")?;
        let Some(dict) = dict else {
            return Ok(Self::default());
        };
        let mut filter_rules = None;
        let mut selection_chars = None;
        let mut hint = None;
        let mut multiselect = None;
        for (tag, obj) in dict {
            let str_tag = tag
                .to_str()
                .context("unexpected non-utf-8 field in opts table")?;
            match str_tag {
                "filter_rules" => {
                    if filter_rules.is_some() {
                        bail!("'filter_rules' supplied more than once");
                    }
                    let fr = FilterRules::parse_obj(obj).context("invalid 'filter_rules'")?;
                    filter_rules = Some(fr);
                }
                "selection_chars" => {
                    if selection_chars.is_some() {
                        bail!("'selection_chars' supplied more than once");
                    }
                    let sel = String::from_object(obj).context("invalid 'selection_chars'")?;
                    selection_chars = Some(sel);
                }
                "hint" => {
                    if hint.is_some() {
                        bail!("'hint' supplied more than once");
                    }
                    let sel_hint = String::from_object(obj).context("invalid 'hint'")?;
                    let parsed = Hint::from_str(&sel_hint)?;
                    hint = Some(parsed);
                }
                "multiselect" => {
                    if multiselect.is_some() {
                        bail!("'multi_select' supplied more than once");
                    }
                    let ms = Multiselect::parse_obj(obj)?;
                    multiselect = ms;
                }

                unk => {
                    bail!("member '{unk}', not recognized");
                }
            }
        }
        let hint = hint.unwrap_or_default();
        let selection_chars = if let Some(sel) = selection_chars {
            validate_provided_selection_chars(&sel, hint)?;
            sel
        } else {
            default_selection_chars()
        };
        Self {
            filter_rules: filter_rules.unwrap_or_default(),
            selection_chars,
            hint,
            multiselect,
        }
        .validate()
    }

    pub(crate) fn validate(self) -> anyhow::Result<Self> {
        if let Some(ms) = self.multiselect {
            if self.selection_chars.contains(ms.commit_char) {
                bail!(
                    "invalid, 'multiselect' and 'selection_chars' overlap with commit_char={}",
                    ms.commit_char
                )
            }
            if self.selection_chars.contains(ms.trigger_char) {
                bail!(
                    "invalid, 'multiselect' and 'selection_chars' overlap with trigger_char={}",
                    ms.trigger_char
                )
            }
        }
        Ok(self)
    }
}

// I know, I know 'parse don't validate', but I don't want to make more structs for this right now
// and parsing is a two-step process where at first all set fields are checked, then those fields
// have validity-dependencies on each other.
fn validate_provided_selection_chars(chars: &str, hint: Hint) -> anyhow::Result<()> {
    let mut set = std::collections::HashSet::with_capacity(chars.len());
    if matches!(hint, Hint::FloatingBigLetter) {
        for ch in chars.chars() {
            if crate::chars::char_to_lines(ch).is_ok() {
                if !set.insert(ch) {
                    bail!("duplicate chars in 'selection_chars'");
                }
            } else {
                bail!("char in 'selection_chars' = {ch} has no big-letter-rendered by this plugin (sorry), chose another one");
            }
        }
    } else {
        for ch in chars.chars() {
            if !set.insert(ch) {
                bail!("duplicate chars in 'selection_chars'");
            }
        }
    }
    Ok(())
}

impl Default for Opts {
    fn default() -> Self {
        if let Some(global) = SETUP_OPTS.get() {
            return global.clone();
        }
        Self {
            filter_rules: FilterRules::default(),
            selection_chars: default_selection_chars(),
            hint: Hint::default(),
            multiselect: None,
        }
    }
}

fn default_selection_chars() -> String {
    "FJDKSLA;CMRUEIWOQP".to_string()
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "test", derive(Eq, PartialEq))]
pub struct FilterRules {
    pub autoselect_one: bool,
    pub include_current_win: bool,
    pub include_floating: bool,
    pub include_unfocusable_windows: bool,
    pub bo: Bo,
    pub file_path_contains: Vec<String>,
    pub file_name_contains: Vec<String>,
}

macro_rules! parse_from_obj_with_err {
    ($dest: ident, $source: expr, $kind: ty) => {
        if $dest.is_some() {
            anyhow::bail!("'{}' supplied more than once", stringify!($dest));
        }
        $dest =
            Some(<$kind>::from_object($source).with_context(|| {
                format!("'{}' not a {}", stringify!($dest), stringify!($kind),)
            })?);
    };
}

impl FilterRules {
    pub(crate) fn parse_obj(object: Object) -> anyhow::Result<Self> {
        let dict = obj_to_dict(object).context("invalid table")?;
        let Some(dict) = dict else {
            return Ok(Self::default());
        };
        let mut autoselect_one = None;
        let mut include_current_win = None;
        let mut include_floating = None;
        let mut include_unfocusable_windows = None;
        let mut bo = None;
        let mut file_path_contains = None;
        let mut file_name_contains = None;
        for (tag, obj) in dict {
            let str_tag = tag
                .to_str()
                .context("unexpected non-utf-8 field in filter_rules")?;
            match str_tag {
                "autoselect_one" => {
                    parse_from_obj_with_err!(autoselect_one, obj, bool);
                }
                "include_current_win" => {
                    parse_from_obj_with_err!(include_current_win, obj, bool);
                }
                "include_floating" => {
                    parse_from_obj_with_err!(include_floating, obj, bool);
                }
                "include_unfocusable_windows" => {
                    parse_from_obj_with_err!(include_unfocusable_windows, obj, bool);
                }
                "bo" => {
                    if bo.is_some() {
                        bail!("'bo' supplied more than once")
                    }
                    bo = Some(Bo::parse_obj(obj)?);
                }
                "file_path_contains" => {
                    parse_from_obj_with_err!(file_path_contains, obj, Vec<String>);
                }
                "file_name_contains" => {
                    parse_from_obj_with_err!(file_name_contains, obj, Vec<String>);
                }

                unk => bail!("failed to parse 'filter_rules' member, '{unk}' not recognized"),
            }
        }
        Ok(Self {
            autoselect_one: autoselect_one.unwrap_or_else(default_true),
            include_current_win: include_current_win.unwrap_or_else(default_true),
            include_floating: include_floating.unwrap_or_default(),
            include_unfocusable_windows: include_unfocusable_windows.unwrap_or_default(),
            bo: bo.unwrap_or_default(),
            file_path_contains: file_path_contains.unwrap_or_default(),
            file_name_contains: file_name_contains.unwrap_or_default(),
        })
    }
}

impl Default for FilterRules {
    fn default() -> Self {
        Self {
            autoselect_one: default_true(),
            include_current_win: default_true(),
            include_floating: default_true(),
            include_unfocusable_windows: false,
            bo: Bo::default(),
            file_path_contains: Vec::default(),
            file_name_contains: Vec::default(),
        }
    }
}

fn obj_to_dict(obj: Object) -> anyhow::Result<Option<Dictionary>> {
    match obj.kind() {
        nvim_oxi::ObjectKind::Nil => Ok(None),
        nvim_oxi::ObjectKind::Array => {
            let arr = Array::from_object(obj)
                .context("Bug, failed to convert checked object to an array")?;
            if !arr.is_empty() {
                bail!("expected a dict, found a non-empty array = {arr:#?}");
            }
            // An empty array is the same as a nil-dict
            Ok(None)
        }
        nvim_oxi::ObjectKind::Dictionary => Ok(Some(
            Dictionary::from_object(obj).context("failed to convert checked object to a dict")?,
        )),
        t => bail!("unexpect object kind {t:?}"),
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "test", derive(Eq, PartialEq))]
pub struct Multiselect {
    pub trigger_char: char,
    pub commit_char: char,
}

impl Multiselect {
    pub(crate) fn parse_obj(object: Object) -> anyhow::Result<Option<Self>> {
        let dict = obj_to_dict(object).context("invalid table")?;
        let Some(dict) = dict else {
            return Ok(None);
        };
        let mut trigger_char = None;
        let mut commit_char = None;
        for (tag, obj) in dict {
            let str_tag = tag
                .to_str()
                .context("unexpected non-utf-8 field in multiselect")?;
            match str_tag {
                "trigger_char" => {
                    parse_from_obj_with_err!(trigger_char, obj, String);
                }
                "commit_char" => {
                    parse_from_obj_with_err!(commit_char, obj, String);
                }
                unk => bail!("failed to parse 'multiselect' member, '{unk}' not recognized"),
            }
        }
        match (trigger_char, commit_char) {
            (Some(trigger), Some(commit)) => {
                let lc_trigger = trigger.to_lowercase();
                let mut lc_trigger_chars = lc_trigger.chars();
                let trigger_char = lc_trigger_chars.next().context(
                    "invalid 'multiselect' configuration, no chars in 'trigger_char'-string",
                )?;
                if lc_trigger_chars.next().is_some() {
                    bail!("invalid 'multiselect' configuration, too many chars in 'trigger_char' string ({trigger}), only one char allowed");
                }
                let lc_commit = commit.to_lowercase();
                let mut lc_commit_chars = lc_commit.chars();
                let commit_char = lc_commit_chars.next().context(
                    "invalid 'multiselect' configuration, no chars in 'commit_char'-string",
                )?;
                if lc_commit_chars.next().is_some() {
                    bail!("invalid 'multiselect' configuration, too many chars in 'commit_char' string ('{commit}'), only one char allowed");
                }
                Ok(Some(Self {
                    trigger_char,
                    commit_char,
                }))
            }
            (Some(_), None) | (None, Some(_)) => {
                bail!("invalid 'multiselect' configuration, only one of 'trigger_char' and 'commit_char' was set, both or none need to be set" );
            }
            (None, None) => Ok(None),
        }
    }
}

#[inline]
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "test", derive(Eq, PartialEq))]
pub struct Bo {
    pub filetype: Vec<String>,
    pub buftype: Vec<String>,
}

impl Default for Bo {
    fn default() -> Self {
        Self {
            filetype: default_fts(),
            buftype: default_bts(),
        }
    }
}

impl Bo {
    fn parse_obj(object: Object) -> anyhow::Result<Self> {
        let dict = obj_to_dict(object).context("invalid table")?;
        let Some(dict) = dict else {
            return Ok(Self::default());
        };
        let mut filetype = None;
        let mut buftype = None;
        for (tag, obj) in dict {
            let str_tag = tag.to_str().context("unexpected non-utf-8 field in bo")?;
            match str_tag {
                "filetype" => {
                    parse_from_obj_with_err!(filetype, obj, Vec<String>);
                }
                "buftype" => {
                    parse_from_obj_with_err!(buftype, obj, Vec<String>);
                }
                unk => bail!("failed to parse 'filter_rules' member, '{unk}' not recognized"),
            }
        }

        Ok(Self {
            filetype: filetype.unwrap_or_default(),
            buftype: buftype.unwrap_or_default(),
        })
    }
}

fn default_fts() -> Vec<String> {
    vec![
        "NvimTree".to_string(),
        "neo-tree".to_string(),
        "notify".to_string(),
        "snacks_notif".to_string(),
    ]
}

fn default_bts() -> Vec<String> {
    vec![
        "terminal".to_string(),
        "nofile".to_string(),
        "prompt".to_string(),
    ]
}

#[derive(Debug)]
pub struct OpenSplitOpts {
    pub(crate) path: String,
    pub(crate) focus_new: bool,
    pub(crate) vertical: bool,
    pub(crate) opts: Opts,
}

impl OpenSplitOpts {
    pub fn parse_obj(object: Object) -> anyhow::Result<Self> {
        let dict = obj_to_dict(object).context("invalid table")?;
        let Some(dict) = dict else {
            bail!("'open_split_opts' needs at least a path");
        };
        let mut path = None;
        let mut focus_new = None;
        let mut vertical = None;
        let mut opts = None;
        for (tag, obj) in dict {
            let str_tag = tag
                .to_str()
                .context("unexpected non-utf-8 field in 'open_split_opts'")?;
            match str_tag {
                "path" => {
                    parse_from_obj_with_err!(path, obj, String);
                }
                "focus_new" => {
                    parse_from_obj_with_err!(focus_new, obj, bool);
                }
                "vertical" => {
                    parse_from_obj_with_err!(vertical, obj, bool);
                }
                "opts" => {
                    opts = Some(Opts::parse_obj(obj)?);
                }
                unk => bail!("failed to parse 'open_split_opts' member, '{unk}' not recognized"),
            }
        }

        Ok(Self {
            path: path.context("'open_split_opts' needs 'path' to be set")?,
            focus_new: focus_new.unwrap_or(true),
            vertical: vertical.unwrap_or_default(),
            opts: opts.unwrap_or_default(),
        })
    }
}

#[derive(Debug)]
pub struct OpenOverOpts {
    pub(crate) path: String,
    pub(crate) focus_new: bool,
    pub(crate) opts: Opts,
}

impl OpenOverOpts {
    pub fn parse_obj(object: Object) -> anyhow::Result<Self> {
        let dict = obj_to_dict(object).context("invalid table")?;
        let Some(dict) = dict else {
            bail!("'open_over_opts' needs at least a path");
        };
        let mut path = None;
        let mut opts = None;
        let mut focus_new = None;
        for (tag, obj) in dict {
            let str_tag = tag
                .to_str()
                .context("unexpected non-utf-8 field in 'open_over_opts'")?;
            match str_tag {
                "path" => {
                    parse_from_obj_with_err!(path, obj, String);
                }
                "opts" => {
                    opts = Some(Opts::parse_obj(obj)?);
                }
                "focus_new" => {
                    parse_from_obj_with_err!(focus_new, obj, bool);
                }
                unk => bail!("failed to parse 'open_over_opts' member, '{unk}' not recognized"),
            }
        }

        Ok(Self {
            path: path.context("'open_split_opts' needs 'path' to be set")?,
            focus_new: focus_new.unwrap_or_else(default_true),
            opts: opts.unwrap_or_default(),
        })
    }
}

pub struct OpenRelativeOpts {
    pub(crate) path: String,
    pub(crate) focus_new: bool,
    pub(crate) relative_chars: String,
    pub(crate) opts: Opts,
}

impl OpenRelativeOpts {
    pub fn parse_obj(object: Object) -> anyhow::Result<Self> {
        let dict = obj_to_dict(object).context("invalid table")?;
        let Some(dict) = dict else {
            bail!("'open_over_opts' needs at least a path");
        };
        let mut path = None;
        let mut opts = None;
        let mut relative_chars = None;
        let mut focus_new = None;
        for (tag, obj) in dict {
            let str_tag = tag
                .to_str()
                .context("unexpected non-utf-8 field in 'open_over_opts'")?;
            match str_tag {
                "path" => {
                    parse_from_obj_with_err!(path, obj, String);
                }
                "opts" => {
                    opts = Some(Opts::parse_obj(obj)?);
                }
                "focus_new" => {
                    parse_from_obj_with_err!(focus_new, obj, bool);
                }
                "relative_chars" => {
                    parse_from_obj_with_err!(relative_chars, obj, String);
                }
                unk => bail!("failed to parse 'open_over_opts' member, '{unk}' not recognized"),
            }
        }

        let opts = opts.unwrap_or_default();
        let relative_chars = if let Some(rel) = relative_chars {
            validate_provided_relative_chars(&rel)?;
            rel
        } else {
            opts.selection_chars.clone()
        };
        // Don't need to check multiselect here, since it's not applicable for open relative
        Ok(Self {
            path: path.context("'open_split_opts' needs 'path' to be set")?,
            focus_new: focus_new.unwrap_or_else(default_true),
            relative_chars,
            opts,
        })
    }
}

// Can theoretically be any char, just need to check for uniqueness
fn validate_provided_relative_chars(chars: &str) -> anyhow::Result<()> {
    let mut set = std::collections::HashSet::with_capacity(chars.len());
    for ch in chars.chars() {
        if !set.insert(ch) {
            bail!("duplicate chars in 'relative_chars'");
        }
    }
    Ok(())
}
