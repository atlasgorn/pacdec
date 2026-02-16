use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use colored::*;
use inquire::Confirm;
use kdl::KdlDocument;

use crate::cli::*;
use crate::kdl_edit::{add_pkgs, remove_pkgs};
use crate::list_pkgs::{collect_documents, get_pkg_diff};
use crate::pacman::sudo_pacman;
use crate::prompts::*;

pub fn search_cmd(cli: &Cli, args: &SearchArgs) -> Result<()> {
    let pkgs: Vec<String>;
    if args.all {
        pkgs = prompt_pkgs_all()?;
    } else if args.explicit {
        pkgs = prompt_pkgs_exp()?;
    } else {
        pkgs = prompt_pkgs_ins()?;
    }
    println!("{:?}", pkgs);
    Ok(())
}

pub fn add_cmd(cli: &Cli, args: &AddArgs) -> Result<()> {
    let documents = collect_documents(&cli.config)?;
    let pkgs = match &args.packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_all()?,
    };
    let pkg_refs: Vec<&str> = pkgs.iter().map(|s: &String| s.as_str()).collect();

    let categories = collect_categories(documents.iter().map(|(_, doc)| doc).collect());
    let category = match args.category.clone() {
        Some(x) => x,
        None => prompt_category(categories.into_iter().collect())?,
    };

    for (_, mut doc) in documents {
        add_pkgs(&mut doc, &category, &pkg_refs)?;
        print!("{doc}");
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

pub fn remove_cmd(cli: &Cli, args: &RemoveArgs) -> Result<()> {
    let mut documents = collect_documents(&cli.config)?;
    let pkgs = match &args.packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_exp()?,
    };
    handle_remove_pkgs(&mut documents, &pkgs)
}

fn handle_remove_pkgs(documents: &mut [(PathBuf, KdlDocument)], pkgs: &[String]) -> Result<()> {
    let doc = &documents[0].0; // TODO: backup all included files as well; backup only changed
    let backup_dir = doc
        .parent()
        .context("config file must have a parent directory")?
        .join(".old");
    backup_file(doc, &backup_dir)?;

    for (file, doc) in documents.iter_mut() {
        remove_pkgs(doc, pkgs)?;
        print!("{doc}");
        fs::write(&file, doc.to_string()).with_context(|| {
            format!("failed to write updated config to file: {}", file.display())
        })?;
    }

    Ok(())
}

pub fn backup_file(path: &Path, backup_dir: &Path) -> Result<()> {
    let file_name = path
        .file_name()
        .context("failed to get file name for backup")?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_");
    let backup_path = backup_dir.join(format!("{}{}", timestamp, file_name.to_string_lossy()));
    fs::copy(path, &backup_path).with_context(|| {
        format!(
            "failed to backup file {} to {}",
            path.display(),
            backup_path.display()
        )
    })?;
    Ok(())
}

pub fn gen_cmd(cli: &Cli) -> Result<()> {
    let mut documents = collect_documents(&cli.config)?; // TODO: pass docs to get_pkg_diff to avoid re-parsing
    let (pkgs_to_add, pkgs_to_remove) = get_pkg_diff(&cli.pacman_log_file, &cli.config)?;
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
                handle_remove_pkgs(&mut documents, &pkgs_to_remove)?;
            }
        }
        Ok(false) => println!("Operation cancelled."),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

pub fn sync_cmd(cli: &Cli) -> Result<()> {
    let (pkgs_to_uninstall, pkgs_to_install) = get_pkg_diff(&cli.pacman_log_file, &cli.config)?;

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
                sudo_pacman(&["-S"], &pkgs_to_install)?;
            }
            if !pkgs_to_uninstall.is_empty() {
                sudo_pacman(&["-Rns"], &pkgs_to_uninstall)?;
            }
        }
        Ok(false) => println!("Operation cancelled."),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
