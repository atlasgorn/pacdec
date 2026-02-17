use anyhow::Result;
use colored::*;
use inquire::Confirm;

use crate::app::App;
use crate::cli::*;
use crate::kdl_edit::{add_pkgs, remove_pkgs, write_changes};
use crate::list_pkgs::get_pkg_diff;
use crate::pacman::sudo_pacman;
use crate::prompts::*;

pub fn install_cmd(app: &mut App, cli: &Cli, args: &InstallArgs) -> Result<()> {
    let pkgs = handle_add_pkgs_cmd(app, cli)?;
    install_pkgs(pkgs)?;
    Ok(())
}

pub fn add_cmd(app: &mut App, cli: &Cli, args: &AddArgs) -> Result<()> {
    handle_add_pkgs_cmd(app, cli)?;
    Ok(())
}

fn handle_add_pkgs_cmd(app: &mut App, cli: &Cli) -> Result<Vec<String>> {
    let (packages, category) = match cli.command {
        Commands::Add(ref args) => (args.packages.clone(), args.category.clone()),
        Commands::Install(ref args) => (args.packages.clone(), args.category.clone()),
        _ => unreachable!(),
    };
    let pkgs = match &packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_all(app)?,
    };

    print!("{}: ", "Adding packages".blue().bold());
    for pkg in &pkgs {
        print!("{pkg} ");
    }
    println!();

    handle_add_pkgs(app, category, &pkgs)?;
    Ok(pkgs)
}

fn handle_add_pkgs(app: &mut App, category: Option<String>, pkgs: &[String]) -> Result<()> {
    let pkg_refs: Vec<&str> = pkgs.iter().map(|s: &String| s.as_str()).collect();
    let category = match category {
        Some(x) => x,
        None => prompt_category(app)?,
    };
    add_pkgs(app, &category, &pkg_refs)?;
    if !app.config.dry_run {
        write_changes(app)?; // TODO: make transactional
    } else {
        for (_, doc) in &app.docs {
            print!("{doc}");
        }
    }
    Ok(())
}

pub fn uninstall_cmd(app: &mut App, args: &UninstallArgs) -> Result<()> {
    let pkgs = match &args.packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_exp(app)?,
    };

    print!("{}: ", "Removing packages".blue().bold());
    for pkg in &pkgs {
        print!("{pkg} ");
    }
    println!();

    handle_remove_pkgs(app, &pkgs)?;
    uninstall_pkgs(pkgs)
}

pub fn remove_cmd(app: &mut App, args: &RemoveArgs) -> Result<()> {
    let pkgs = match &args.packages {
        Some(pkgs) => pkgs.clone(),
        None => prompt_pkgs_exp(app)?,
    };

    print!("{}: ", "Removing packages".blue().bold());
    for pkg in &pkgs {
        print!("{pkg} ");
    }
    println!();

    handle_remove_pkgs(app, &pkgs)
}

fn handle_remove_pkgs(app: &mut App, pkgs: &[String]) -> Result<()> {
    remove_pkgs(app, pkgs)?;
    if !app.config.dry_run {
        write_changes(app)?; // TODO: make transactional
    } else {
        for (_, doc) in &app.docs {
            print!("{doc}");
        }
    }
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

pub fn search_cmd(app: &mut App, args: &SearchArgs) -> Result<()> {
    let pkgs = match args {
        SearchArgs { all: true, .. } => prompt_pkgs_all(app)?,
        SearchArgs { explicit: true, .. } => prompt_pkgs_exp(app)?,
        _ => prompt_pkgs_ins(app)?,
    };

    for pkg in pkgs {
        println!("{}", pkg);
    }
    Ok(())
}

fn uninstall_pkgs(pkgs_to_uninstall: Vec<String>) -> Result<()> {
    sudo_pacman(&["-Rns"], &pkgs_to_uninstall)?;
    Ok(())
}

fn install_pkgs(pkgs_to_install: Vec<String>) -> Result<()> {
    sudo_pacman(&["-S"], &pkgs_to_install)?;
    Ok(())
}
