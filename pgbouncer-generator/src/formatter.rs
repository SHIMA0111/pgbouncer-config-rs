use std::fmt::Write;

use colored::Colorize;
use pgbouncer_config::utils::diff::Diff;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DisplayOptions {
    decoration: bool,
    show_same: bool,
    max_diff_depth: usize,
}

impl DisplayOptions {
    pub(crate) fn new(decoration: bool, show_same: bool, max_diff_depth: usize) -> Self {
        Self {
            decoration,
            show_same,
            max_diff_depth,
        }
    }
}

pub(crate) fn format_diff(diff: &Diff, opt: DisplayOptions) -> String {
    let mut output = String::new();
    format_diff_inner(diff, opt, 0, &mut output, None);
    output
}

fn format_diff_inner(
    diff: &Diff,
    opt: DisplayOptions,
    depth: usize,
    output: &mut String,
    current_key: Option<&str>,
) {
    if opt.max_diff_depth != 0 && depth > opt.max_diff_depth {
        let _ = writeln!(output, "{}...(max depth reached)", indent(depth));
        return;
    }

    match diff {
        Diff::Same { value } => {
            if opt.show_same {
                line_same(output, depth, current_key, value);
            }
        },
        Diff::Unevaluated => {
            line_meta(output, depth, current_key, "<unevaluated>", opt.decoration);
        },
        Diff::Changed { old, new } => {
            line_minus(output, depth, current_key, old, opt.decoration);
            line_plus(output, depth, current_key, new, opt.decoration);
        },
        Diff::Added { new } => {
            line_plus(output, depth, current_key, new, opt.decoration);
        },
        Diff::Removed { old } => {
            line_minus(output, depth, current_key, old, opt.decoration);
        },
        Diff::Object { fields } => {
            if let Some(k) = current_key {
                header(output, depth, &format!("{{{}}}", k), opt.decoration);
            }
            for (k, v) in fields {
                if matches!(v, Diff::Same { .. }) && !opt.show_same {
                    continue;
                }
                format_diff_inner(v, opt, depth + 1, output, Some(k));
            }
        },
        Diff::Array { items } => {
            if let Some(k) = current_key {
                header(output, depth, &format!("[{}]", k), opt.decoration);
            }
            for (idx, v) in items {
                match v {
                    Diff::Same { value } => {
                        if opt.show_same {
                            line_same(output, depth + 1, Some(&format!("[{}]", idx)), value);
                        }
                    }
                    Diff::Changed { old, new } => {
                        line_minus(output, depth + 1, Some(&format!("[{}]", idx)), old, opt.decoration);
                        line_plus(output, depth + 1, Some(&format!("[{}]", idx)), new, opt.decoration);
                    }
                    Diff::Added { new } => {
                        line_plus(output, depth + 1, Some(&format!("[{}]", idx)), new, opt.decoration);
                    }
                    Diff::Removed { old } => {
                        line_minus(output, depth + 1, Some(&format!("[{}]", idx)), old, opt.decoration);
                    }
                    Diff::Object { .. } | Diff::Array { .. } | Diff::Unevaluated => {
                        format_diff_inner(v, opt, depth + 1, output, Some(&format!("[{}]", idx)));
                    }
                }
            }
        }
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn header(out: &mut String, depth: usize, title: &str, decoration: bool) {
    let title = if decoration {
        title.cyan().bold().to_string()
    } else {
        title.to_string()
    };
    let _ = writeln!(out, "{}{}", indent(depth), title);
}

fn line_plus(out: &mut String, depth: usize, key: Option<&str>, line: &str, decoration: bool) {
    let prefix = if decoration {
        "+".green().to_string()
    } else {
        "+".to_string()
    };
    let key_s = key.map(|k| format!("{}: ", k)).unwrap_or_default();
    let _ = writeln!(out, "{}{}{}{}", indent(depth), prefix, key_s, line);
}

fn line_minus(out: &mut String, depth: usize, key: Option<&str>, line: &str, decoration: bool) {
    let prefix = if decoration {
        "-".red().to_string()
    } else {
        "-".to_string()
    };
    let key_s = key.map(|k| format!("{}: ", k)).unwrap_or_default();
    let _ = writeln!(out, "{}{}{}{}", indent(depth), prefix, key_s, line);
}

fn line_meta(out: &mut String, depth: usize, key: Option<&str>, line: &str, decoration: bool) {
    let prefix = if decoration {
        "~".cyan().to_string()
    } else {
        "~".to_string()
    };
    let key_s = key.map(|k| format!("{}: ", k)).unwrap_or_default();
    let _ = writeln!(out, "{}{}{}{}", indent(depth), prefix, key_s, line);
}

fn line_same(out: &mut String, depth: usize, key: Option<&str>, line: &str) {
    let key_s = key.map(|k| format!("{}: ", k)).unwrap_or_default();
    let _ = writeln!(out, "{} {}{}", indent(depth), key_s, line);
}