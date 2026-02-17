use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use colored::*;
use inquire::Confirm;
use kdl::KdlDocument;

use crate::app::App;
use crate::cli::*;
use crate::kdl_edit::{add_pkgs, remove_pkgs};
use crate::list_pkgs::get_pkg_diff;
use crate::pacman::sudo_pacman;
use crate::prompts::*;

pub fn search_cmd(app: &mut App, args: &SearchArgs) -> Result<()> {
    let pkgs: Vec<String>;
    if args.all {
        pkgs = prompt_pkgs_all(app)?;
    } else if args.explicit {
        pkgs = prompt_pkgs_exp(app)?;
    } else {
        pkgs = prompt_pkgs_ins(app)?;
    }
    println!("{:?}", pkgs);
    Ok(())
}

pub fn install_cmd(app: &mut App, cli: &Cli, args: &InstallArgs) -> Result<()> {
    let pkgs = handle_add_pkgs_cmd(app, cli)?;
    install_pkgs(pkgs)?;
    Ok(())
}

pub fn add_cmd(app: &mut App, cli: &Cli, args: &AddArgs) -> Result<()> {
    handle_add_pkgs_cmd(app, cli)?;
    Ok(())
}

fn handle_add_pkgs_cmd(app: &mut App, cli: &Cli) -> Result<Vec<String>, anyhow::Error> {
    let (packages, category) = match cli.command {
        Commands::Add(ref args) => (args.packages.clone(), args.category.clone()),
        Commands::Install(ref args) => (args.packages.clone(), args.category.clone()),
        _ => unreachable!(),
    };
    let pkgs = match &packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_all(app)?,
    };
    handle_add_pkgs(app, category, &pkgs)?;
    Ok(pkgs)
}

fn handle_add_pkgs(
    app: &mut App,
    category: Option<String>,
    pkgs: &[String],
) -> Result<(), anyhow::Error> {
    let pkg_refs: Vec<&str> = pkgs.iter().map(|s: &String| s.as_str()).collect();
    let categories = collect_categories(app.docs.iter().map(|(_, doc)| doc).collect());
    let category = match category {
        Some(x) => x,
        None => prompt_category(categories.into_iter().collect())?,
    };
    for (_, doc) in &mut app.docs {
        add_pkgs(doc, &category, &pkg_refs)?;
        if app.config.dry_run {
            print!("{doc}");
        }
    }
    if !app.config.dry_run {
        write_changes(app)?;
    }
    Ok(())
}

pub fn collect_categories(documents: Vec<&KdlDocument>) -> HashSet<String> {
    let mut categories = HashSet::new();

    for doc in documents {
        let mut stack = vec![doc.nodes()];
        while let Some(nodes) = stack.pop() {
            for node in nodes {
                if node.name().value().starts_with("cat:") {
                    let cat_name = node.name().value().trim_start_matches("cat:").to_string();
                    categories.insert(cat_name);
                }
                if let Some(children) = node.children() {
                    stack.push(children.nodes());
                }
            }
        }
    }
    categories
}

pub fn uninstall_cmd(app: &mut App, args: &UninstallArgs) -> Result<()> {
    let pkgs = match &args.packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_exp(app)?,
    };
    handle_remove_pkgs(app, &pkgs)?;
    uninstall_pkgs(pkgs)?;
    Ok(())
}

pub fn remove_cmd(app: &mut App, args: &RemoveArgs) -> Result<()> {
    let pkgs = match &args.packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_exp(app)?,
    };
    handle_remove_pkgs(app, &pkgs)
}

fn handle_remove_pkgs(app: &mut App, pkgs: &[String]) -> Result<()> {
    for (_, doc) in &mut app.docs {
        remove_pkgs(doc, pkgs)?;
        if app.config.dry_run {
            print!("{doc}");
        }
    }
    if !app.config.dry_run {
        write_changes(app)?;
    }
    Ok(())
}

pub fn write_changes(app: &mut App) -> Result<()> {
    let text_docs: Vec<(PathBuf, String)> = app
        .docs
        .iter()
        .map(|(file, doc)| (file.clone(), doc.to_string()))
        .collect();

    let changed_files: Vec<(PathBuf, String)> = text_docs
        .into_iter()
        .filter(|(file, new_content)| {
            if let Ok(current_content) = fs::read_to_string(file) {
                current_content != *new_content
            } else {
                true
            }
        })
        .collect();

    for (file, new_content) in changed_files {
        backup(app, &file)?;

        fs::write(&file, new_content)?;
    }

    Ok(())
}

fn backup(app: &mut App, path: &PathBuf) -> Result<(), anyhow::Error> {
    let backup_dir = path
        .parent()
        .context("config file must have a parent directory")?
        .join(app.config.backup_dir.clone());
    fs::create_dir_all(&backup_dir)?;

    let file_name = path
        .file_name()
        .context("failed to get file name for backup")?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("{}_{}", timestamp, file_name.to_string_lossy()));

    fs::copy(path, &backup_path).with_context(|| {
        format!(
            "failed to backup file {} to {}",
            path.display(),
            backup_path.display()
        )
    })?;
    Ok(())
}

pub fn gen_cmd(app: &mut App) -> Result<()> {
    let (pkgs_to_add, pkgs_to_remove) = get_pkg_diff(&mut app.docs, &app.config.pacman_log_file)?;
    if pkgs_to_add.is_empty() && pkgs_to_remove.is_empty() {
        println!(
            "{}",
            "Packages are in sync, nothing to generate".blue().bold()
        );
        return Ok(());
    }
    if !pkgs_to_add.is_empty() {
        println!(
            "{} {}:",
            "\nPackages to add to config".blue().bold(),
            pkgs_to_add.len().to_string().green()
        );
        for pkg in &pkgs_to_add {
            print!("{pkg} ");
        }
        println!();
    }
    if !pkgs_to_remove.is_empty() {
        println!(
            "\n{} {}:",
            "Packages to remove from config".blue().bold(),
            pkgs_to_remove.len().to_string().red()
        );
        for pkg in &pkgs_to_remove {
            print!("{pkg} ");
        }
    }
    println!();
    match Confirm::new("Proceed?").with_default(true).prompt() {
        Ok(true) => {
            if !pkgs_to_remove.is_empty() {
                handle_remove_pkgs(app, &pkgs_to_remove)?;
            }
            if !pkgs_to_add.is_empty() {
                handle_add_pkgs(app, Some(app.config.default_category.clone()), &pkgs_to_add)?;
            }
        }
        Ok(false) => println!("Operation cancelled."),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

pub fn sync_cmd(app: &mut App) -> Result<()> {
    let (pkgs_to_uninstall, pkgs_to_install) =
        get_pkg_diff(&mut app.docs, &app.config.pacman_log_file)?;

    if pkgs_to_uninstall.is_empty() && pkgs_to_install.is_empty() {
        println!("{}", "Packages are in sync, nothing to do".blue().bold());
        return Ok(());
    }
    if !pkgs_to_install.is_empty() {
        println!(
            "\n{} {}:",
            "Packages to intall".blue().bold(),
            pkgs_to_install.len().to_string().green()
        );
        for pkg in &pkgs_to_install {
            print!("{pkg} ");
        }
        println!();
    }
    if !pkgs_to_uninstall.is_empty() {
        println!(
            "{} {}:",
            "\nPackages to uninstall".blue().bold(),
            pkgs_to_uninstall.len().to_string().red()
        );
        for pkg in &pkgs_to_uninstall {
            print!("{pkg} ");
        }
    }
    println!();
    match Confirm::new("Proceed?").with_default(true).prompt() {
        Ok(true) => {
            if !pkgs_to_install.is_empty() {
                install_pkgs(pkgs_to_install)?;
            }
            if !pkgs_to_uninstall.is_empty() {
                uninstall_pkgs(pkgs_to_uninstall)?;
            }
        }
        Ok(false) => println!("Operation cancelled."),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

fn uninstall_pkgs(pkgs_to_uninstall: Vec<String>) -> Result<(), anyhow::Error> {
    sudo_pacman(&["-Rns"], &pkgs_to_uninstall)?;
    Ok(())
}

fn install_pkgs(pkgs_to_install: Vec<String>) -> Result<(), anyhow::Error> {
    sudo_pacman(&["-S"], &pkgs_to_install)?;
    Ok(())
}
