use nvim_oxi::{Array, Dictionary};
use nvim_winpick_core::{
    Hint, OpenOverOpts, OpenRelativeOpts, OpenSplitOpts, Opts, safe_parse_opts,
};

#[nvim_oxi::test]
fn supply_default_opts() {
    let safe = safe_parse_opts(None).unwrap();
    assert_eq!(Opts::default(), safe);
}

#[nvim_oxi::test]
fn set_global_opts_replaces_default() {
    let safe = safe_parse_opts(None).unwrap();
    assert_eq!(Opts::default(), safe);
    let mut dict = Dictionary::new();
    dict.insert("selection_chars", "abcdefg");
    let obj = dict.into();
    nvim_winpick_core::setup(Some(obj));
    let modified_defaults = Opts {
        selection_chars: "abcdefg".to_string(),
        ..Opts::default()
    };
    assert_eq!(Opts::default(), modified_defaults);
}

#[nvim_oxi::test]
fn set_global_opts_fails_if_tried_twice() {
    let mut dict = Dictionary::new();
    dict.insert("selection_chars", "abcdefg");
    let obj = dict.into();
    nvim_winpick_core::setup(Some(obj));
    let modified_defaults = Opts {
        selection_chars: "abcdefg".to_string(),
        ..Opts::default()
    };
    assert_eq!(Opts::default(), modified_defaults);
    let mut dict = Dictionary::new();
    dict.insert("selection_chars", "loi");
    nvim_winpick_core::setup(Some(dict.into()));

    // Not updated
    assert_eq!(Opts::default(), modified_defaults);
}

#[nvim_oxi::test]
fn rejects_duplicate_selection_chars() {
    let mut dict = Dictionary::new();
    dict.insert("selection_chars", "abcdefg");
    let obj = dict.into();
    let safe = safe_parse_opts(Some(obj));
    assert!(safe.is_some());
    let mut dict = Dictionary::new();
    dict.insert("selection_chars", "abcdeff");
    let obj = dict.into();
    let safe = safe_parse_opts(Some(obj));
    assert!(safe.is_none());
}

#[nvim_oxi::test]
fn rejects_unrendered_characters_on_floating_big_letter_allows_on_floating_letter() {
    // Check ö, will also implicitly check that the default is floating-big-letter
    let mut dict = Dictionary::new();
    dict.insert("selection_chars", "ö");
    let obj = dict.into();
    let safe = safe_parse_opts(Some(obj));
    assert!(safe.is_none());
    let mut dict = Dictionary::new();
    dict.insert("selection_chars", "ö");
    dict.insert("hint", "floating-letter");
    let obj = dict.into();
    let safe = safe_parse_opts(Some(obj));
    assert!(safe.is_some());
}

#[nvim_oxi::test]
fn unknown_opts_field_rejected() {
    let mut dict = Dictionary::new();
    dict.insert("unk_field", "abcdefg");
    let obj = dict.into();
    let safe = safe_parse_opts(Some(obj));
    assert!(safe.is_none());
}

#[nvim_oxi::test]
fn valid_filter_rules_field_accepted() {
    let mut filter_rules = Dictionary::new();
    filter_rules.insert("autoselect_one", false);
    let mut dict = Dictionary::new();
    dict.insert("filter_rules", filter_rules);
    let obj = dict.into();
    let parsed = safe_parse_opts(Some(obj)).unwrap();
    assert!(!parsed.filter_rules.autoselect_one);
}

#[nvim_oxi::test]
fn unknown_filter_rules_field_rejected() {
    let mut filter_rules = Dictionary::new();
    filter_rules.insert("autoselect_two", true);
    let mut dict = Dictionary::new();
    dict.insert("filter_rules", filter_rules);
    let obj = dict.into();
    assert!(safe_parse_opts(Some(obj)).is_none());
}

#[nvim_oxi::test]
fn valid_filter_rules_bo_field_accepted() {
    let mut bo = Dictionary::new();
    let mut arr = Array::new();
    arr.push("rust");
    bo.insert("filetype", arr);
    let mut filter_rules = Dictionary::new();
    filter_rules.insert("bo", bo.clone());
    let mut dict = Dictionary::new();
    dict.insert("filter_rules", filter_rules);
    let obj = dict.into();
    let opts = safe_parse_opts(Some(obj)).unwrap();
    assert_eq!(vec!["rust".to_string()], opts.filter_rules.bo.filetype);
}

#[nvim_oxi::test]
fn unknown_filter_rules_bo_field_rejected() {
    let mut bo = Dictionary::new();
    let mut arr = Array::new();
    arr.push("rust");
    bo.insert("filetypes", arr);
    let mut filter_rules = Dictionary::new();
    filter_rules.insert("bo", bo.clone());
    let mut dict = Dictionary::new();
    dict.insert("filter_rules", filter_rules);
    let obj = dict.into();
    assert!(safe_parse_opts(Some(obj)).is_none());
}

#[nvim_oxi::test]
fn valid_hints_accepted() {
    let mut dict = Dictionary::new();
    dict.insert("hint", "floating-letter");
    let obj = dict.into();
    let opts = safe_parse_opts(Some(obj)).unwrap();
    assert!(matches!(opts.hint, Hint::FloatingLetter));
    let mut dict = Dictionary::new();
    dict.insert("hint", "floating-big-letter");
    let obj = dict.into();
    let opts = safe_parse_opts(Some(obj)).unwrap();
    assert!(matches!(opts.hint, Hint::FloatingBigLetter));
}

#[nvim_oxi::test]
fn unknown_hint_rejected() {
    let mut dict = Dictionary::new();
    dict.insert("hint", "floating-small-letter");
    let obj = dict.into();
    assert!(safe_parse_opts(Some(obj)).is_none());
}

#[nvim_oxi::test]
fn unknown_multiselect_field_rejected() {
    let mut multiselect = Dictionary::new();
    multiselect.insert("trigger", "c");
    let mut dict = Dictionary::new();
    dict.insert("multiselect", Some(multiselect));
    let obj = dict.into();
    assert!(safe_parse_opts(Some(obj)).is_none());
}

#[nvim_oxi::test]
fn valid_multiselect_accepted() {
    let mut multiselect = Dictionary::new();
    multiselect.insert("trigger_char", "m");
    multiselect.insert("commit_char", "c");
    let mut dict = Dictionary::new();
    dict.insert("multiselect", Some(multiselect));
    let obj = dict.into();
    let opts = safe_parse_opts(Some(obj)).unwrap();
    let multi = opts.multiselect.unwrap();
    assert_eq!('c', multi.commit_char);
    assert_eq!('m', multi.trigger_char);
}

#[nvim_oxi::test]
fn incomplete_multiselect_rejected() {
    let mut multiselect = Dictionary::new();
    multiselect.insert("commit_char", "c");
    let mut dict = Dictionary::new();
    dict.insert("multiselect", Some(multiselect));
    let obj = dict.into();
    assert!(safe_parse_opts(Some(obj)).is_none());

    let mut multiselect = Dictionary::new();
    multiselect.insert("trigger_char", "c");
    let mut dict = Dictionary::new();
    dict.insert("multiselect", Some(multiselect));
    let obj = dict.into();
    assert!(safe_parse_opts(Some(obj)).is_none());
}

#[nvim_oxi::test]
fn open_over_opts_invalid_if_empty() {
    let dict = Dictionary::new();
    let obj = dict.into();
    let res = OpenOverOpts::parse_obj(obj);
    assert!(res.is_err());
}

#[nvim_oxi::test]
fn open_over_opts_valid_with_path() {
    let mut dict = Dictionary::new();
    dict.insert("path", "my-path");
    let obj = dict.into();
    let res = OpenOverOpts::parse_obj(obj);
    assert!(res.is_ok());
}

#[nvim_oxi::test]
fn open_split_opts_invalid_if_empty() {
    let dict = Dictionary::new();
    let obj = dict.into();
    let res = OpenSplitOpts::parse_obj(obj);
    assert!(res.is_err());
}

#[nvim_oxi::test]
fn open_split_opts_valid_with_path() {
    let mut dict = Dictionary::new();
    dict.insert("path", "my-path");
    let obj = dict.into();
    let res = OpenSplitOpts::parse_obj(obj);
    assert!(res.is_ok());
}

#[nvim_oxi::test]
fn open_relative_opts_invalid_if_empty() {
    let dict = Dictionary::new();
    let obj = dict.into();
    let res = OpenRelativeOpts::parse_obj(obj);
    assert!(res.is_err());
}

#[nvim_oxi::test]
fn open_relative_opts_valid_with_path() {
    let mut dict = Dictionary::new();
    dict.insert("path", "my-path");
    let obj = dict.into();
    let res = OpenRelativeOpts::parse_obj(obj);
    assert!(res.is_ok());
}

#[nvim_oxi::test]
fn open_relative_opts_allows_any_char_in_relative_chars() {
    let mut dict = Dictionary::new();
    dict.insert("path", "my-path");
    dict.insert("relative_chars", "ö");
    let obj = dict.into();
    let res = OpenRelativeOpts::parse_obj(obj);
    assert!(res.is_ok());
}

#[nvim_oxi::test]
fn open_relative_opts_rejects_duplicates_in_relative_chars() {
    let mut dict = Dictionary::new();
    dict.insert("path", "my-path");
    dict.insert("relative_chars", "aa");
    let obj = dict.into();
    let res = OpenRelativeOpts::parse_obj(obj);
    assert!(res.is_err());
}
