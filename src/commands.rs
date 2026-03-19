use anyhow::{Result, bail};
use colored::*;
use inquire::Confirm;
use rayon::prelude::*;

use crate::app::App;
use crate::cli::*;
use crate::config::Config;
use crate::kdl_edit::{add_pkgs, apply_dec_changes, remove_pkgs};
use crate::list_pkgs::get_pkg_diff;
use crate::packages::{Package, PackageJoin};
use crate::pacman::{check_pkg_exists, sudo_pacman};
use crate::prompts::*;

pub fn add_cmd(app: &mut App, cli: Cli, and_install: bool) -> Result<()> {
    let pkgs = {
        let (packages, category, tags) = match cli.command {
            Commands::Add(args) => (args.packages, args.category, args.tags),
            Commands::Install(args) => (args.packages, args.category, args.tags),
            _ => unreachable!(),
        };

        let categories = collect_categories(app);
        if let Some(ref cat) = category
            && !categories.contains(cat)
        // TODO: Check for similarly named categories and for categories excluded due to rules
        {
            bail!("category '{}' not found", cat)
        }

        if let Some(pkgs) = &packages {
            let missing: Vec<String> = pkgs
                .par_iter()
                .filter(|pkg| !check_pkg_exists(&app.config, &pkg.to_string()))
                .map(|pkg| pkg.to_string())
                .collect();

            if !missing.is_empty() {
                let pkg_list = missing.join(", ");
                bail!("package(s) not found in repositories: {}", pkg_list);
            }
        }

        let mut pkgs = match &packages {
            Some(pkgs) => pkgs.clone(),
            None => prompt_pkgs_all(app)?,
        };

        if let Some(ref tags) = tags {
            for pkg in pkgs.iter_mut() {
                pkg.tags = tags.clone();
            }
        }

        println!("{}: {}", "Adding packages".blue().bold(), pkgs.join(" "));

        let category = match category {
            Some(x) => x,
            None => prompt_category(app)?,
        };

        add_pkgs(app, &category, &pkgs)?;

        pkgs
    };

    if and_install {
        install_pkgs(&app.config, pkgs)?;
    }

    apply_dec_changes(app)?;

    Ok(())
}

pub fn remove_cmd(app: &mut App, cli: Cli, and_uninstall: bool) -> Result<()> {
    let pkgs = match match cli.command {
        Commands::Remove(args) => args.packages,
        Commands::Uninstall(args) => args.packages,
        _ => unreachable!(),
    } {
        Some(pkgs) => pkgs.to_owned(),
        None => prompt_pkgs_exp(app)?,
    };

    println!("{} {}", "Removing packages:".blue().bold(), pkgs.join(" "));

    remove_pkgs(app, &pkgs)?;

    if and_uninstall {
        uninstall_pkgs(&app.config, pkgs)?;
    };

    apply_dec_changes(app)?;

    Ok(())
}

pub fn gen_cmd(app: &mut App) -> Result<()> {
    let (pkgs_to_add, pkgs_to_remove) = get_pkg_diff(app)?;
    if pkgs_to_add.is_empty() && pkgs_to_remove.is_empty() {
        println!(
            "{}",
            "Packages are in sync, nothing to generate".blue().bold()
        );
        return Ok(());
    }
    if !pkgs_to_add.is_empty() {
        println!(
            "\n{} {}:",
            "Packages to add to config".blue().bold(),
            pkgs_to_add.len().to_string().green()
        );
        println!("{}", pkgs_to_add.join(" "));
    }
    if !pkgs_to_remove.is_empty() {
        println!(
            "\n{} {}:",
            "Packages to remove from config".blue().bold(),
            pkgs_to_remove.len().to_string().red()
        );
        println!("{}", pkgs_to_remove.join(" "));
    }
    println!();

    if !Confirm::new("Proceed?").with_default(true).prompt()? {
        println!("Operation cancelled");
        return Ok(());
    }

    if !pkgs_to_remove.is_empty() {
        remove_pkgs(app, &pkgs_to_remove)?;
    }
    if !pkgs_to_add.is_empty() {
        add_pkgs(app, &(app.config.default_category.clone()), &pkgs_to_add)?;
    }

    apply_dec_changes(app)?;

    Ok(())
}

pub fn sync_cmd(app: &App) -> Result<()> {
    let (pkgs_to_uninstall, pkgs_to_install) = get_pkg_diff(app)?;

    if pkgs_to_uninstall.is_empty() && pkgs_to_install.is_empty() {
        println!("{}", "Packages are in sync, nothing to do".blue().bold());
        return Ok(());
    }
    if !pkgs_to_install.is_empty() {
        println!(
            "\n{} {}:",
            "Packages to install".blue().bold(),
            pkgs_to_install.len().to_string().green()
        );
        println!("{}", pkgs_to_install.join(" "));
    }
    if !pkgs_to_uninstall.is_empty() {
        println!(
            "\n{} {}:",
            "Packages to uninstall".blue().bold(),
            pkgs_to_uninstall.len().to_string().red()
        );
        println!("{}", pkgs_to_uninstall.join(" "));
    }
    println!();

    if !Confirm::new("Proceed?").with_default(true).prompt()? {
        println!("Operation cancelled");
        return Ok(());
    }

    if !pkgs_to_install.is_empty() {
        install_pkgs(&app.config, pkgs_to_install)?;
    }
    if !pkgs_to_uninstall.is_empty() {
        uninstall_pkgs(&app.config, pkgs_to_uninstall)?;
    }

    Ok(())
}

pub fn search_cmd(app: &App, args: &SearchArgs) -> Result<()> {
    let pkgs = match args {
        SearchArgs { all: true, .. } => prompt_pkgs_all(app)?,
        SearchArgs { explicit: true, .. } => prompt_pkgs_exp(app)?,
        _ => prompt_pkgs_ins(app)?,
    };

    print!("{}", pkgs.join("\n"));
    Ok(())
}

fn uninstall_pkgs(cfg: &Config, pkgs: Vec<Package>) -> Result<()> {
    let pkgs: Vec<String> = pkgs.into_iter().map(|pkg| pkg.to_string()).collect();
    sudo_pacman(cfg, &["-Rns"], &pkgs)?;
    Ok(())
}

fn install_pkgs(cfg: &Config, pkgs: Vec<Package>) -> Result<()> {
    let pkgs: Vec<String> = pkgs.into_iter().map(|pkg| pkg.to_string()).collect();
    sudo_pacman(cfg, &["-S", "--asexplicit"], &pkgs)?; // --asexplicit does not work with yay
    Ok(())
}
