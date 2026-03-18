use std::path::{Path, PathBuf};
use std::{fmt, fs};

use anyhow::{Context, Result};

use anyhow::bail;
use colored::*;
use kdl::{FormatConfig, KdlNode};
use similar::{ChangeTag, TextDiff};

use crate::app::App;
use crate::config::{BackupMode, Config};
use crate::packages::{Category, Package};

pub fn add_pkgs(app: &mut App, category: &Category, pkgs: &[Package]) -> Result<()> {
    let category_name = format!("cat:{}", category.name);
    let mut stack = Vec::new();
    for (_, doc) in &mut app.docs {
        stack.push((doc.nodes_mut(), Vec::<String>::new()));
    }
    let mut cat_count = 0;

    while let Some((nodes, path)) = stack.pop() {
        for node in nodes {
            let node_name = node.name().value().to_string();
            if node_name == category_name && (category.path.is_empty() || path == category.path) {
                cat_count += 1;
                if cat_count > 1 {
                    continue;
                }

                let indent = node
                    .format()
                    .map(|format| format.leading.len() / 4)
                    .unwrap_or(0);

                node.ensure_children()
                    .nodes_mut()
                    .extend(pkgs.iter().map(|pkg| {
                        let mut new_node: KdlNode = pkg.clone().into();
                        new_node.autoformat_config(
                            &FormatConfig::builder().indent_level(indent + 1).build(),
                        ); // TODO: There should be a better way to do this
                        new_node
                    }));
            }
            if let Some(children) = node.children_mut() {
                let new_path = if node_name.starts_with("cat:") {
                    let mut new_path = path.clone();
                    new_path.push(node_name.trim_start_matches("cat:").to_string());
                    new_path
                } else {
                    path.clone()
                };
                stack.push((children.nodes_mut(), new_path));
            }
        }
    }
    if cat_count > 1 {
        println!("{}", format!(
            "category {category_name} exists in several places, adding package(s) only to 1st occurrence") // TODO: prompt to choose occurrence 
            .yellow().bold()
        );
    } else if cat_count == 0 {
        bail!("no such category \"{category_name}\" exists");
    }

    Ok(())
}

pub fn remove_pkgs(app: &mut App, pkgs: &[Package]) -> Result<()> {
    let mut stack = Vec::new();
    for (_, doc) in &mut app.docs {
        stack.push(doc.nodes_mut());
    }
    let comment = true; // TODO: get from config

    while let Some(nodes) = stack.pop() {
        if !comment {
            nodes.retain(|node| !pkgs.iter().any(|pkg| pkg.name == node.name().value()));
        }
        for node in nodes {
            if comment {
                if pkgs.iter().any(|pkg| pkg.name == node.name().value()) {
                    node.format_mut()
                        .expect("every node should have format")
                        .leading += "/- ";
                }
                if let Some(children) = node.children_mut() {
                    stack.push(children.nodes_mut());
                }
            }
        }
    }

    Ok(())
}

fn print_diff(path: &Path, old: &str, new: &str) {
    println!("{}", path.display());

    let diff = TextDiff::from_lines(old, new);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^80}", "-");
        }

        for op in group {
            for change in diff.iter_inline_changes(op) {
                let sign_style = match change.tag() {
                    ChangeTag::Delete => "-".red().bold(),
                    ChangeTag::Insert => "+".green().bold(),
                    ChangeTag::Equal => " ".into(),
                };

                print!(
                    "{}{}{}{}",
                    format_line(change.old_index()),
                    format_line(change.new_index()),
                    "|".blue(),
                    sign_style
                );

                fn format_line(line: Option<usize>) -> ColoredString {
                    line.map(|idx| format!("{:<4}", idx + 1))
                        .unwrap_or_else(|| "    ".to_string())
                        .blue()
                }

                for (_, value) in change.iter_strings_lossy() {
                    let styled = match change.tag() {
                        ChangeTag::Delete => value.red(),
                        ChangeTag::Insert => value.green(),
                        ChangeTag::Equal => value.dimmed(),
                    };
                    print!("{}", styled);
                }
            }
        }
    }
    println!();
}

pub fn apply_dec_changes(app: &App) -> Result<()> {
    for (file, doc) in &app.docs {
        let new_content = doc.to_string();
        let current_content = fs::read_to_string(file).unwrap_or_default();

        if current_content == new_content {
            continue;
        }

        if app.config.dry_run || app.config.verbose {
            print_diff(file, &current_content, &new_content);
        }

        if app.config.dry_run {
            continue;
        }

        backup(&app.config, file)?;
        fs::write(file, new_content)?;
    }

    Ok(())
}
fn backup(cfg: &Config, path: &PathBuf) -> Result<()> {
    match cfg.backup.mode {
        BackupMode::Basic => {
            let backup_dir = path
                .parent()
                .context("config file must have a parent directory")?
                .join(cfg.backup.dir.clone());
            fs::create_dir_all(&backup_dir)?;

            let file_name = path
                .file_name()
                .context("failed to get file name for backup")?;
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_path =
                backup_dir.join(format!("{}_{}", timestamp, file_name.to_string_lossy()));

            fs::copy(path, &backup_path).with_context(|| {
                format!(
                    "failed to backup file {} to {}",
                    path.display(),
                    backup_path.display()
                )
            })?;
            Ok(())
        }
        BackupMode::Git => todo!("backup mode not implemented"), // TODO: implement git mode
        BackupMode::Off => Ok(()),
    }
}
