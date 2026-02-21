use anyhow::Result;
use duct::cmd;
use inquire::Select;
use kdl::KdlNode;
use std::collections::HashSet;

use crate::{
    app::App,
    packages::{Category, Package},
};

pub fn prompt_category(app: &App) -> Result<Category> {
    let mut categories: Vec<Category> = collect_categories(app).into_iter().collect();
    let default_cat = &app.config.default_category;
    categories.sort_by_key(|c| c.full_path());
    if categories.contains(default_cat) {
        categories.retain(|cat| cat != default_cat);
        categories.insert(0, default_cat.clone());
    }
    match Select::new("input category", categories).prompt() {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(e.into()),
    }
}

pub fn collect_categories(app: &App) -> HashSet<Category> {
    let mut categories = HashSet::new();
    let mut path = Vec::new();

    for (_, doc) in &app.docs {
        traverse_nodes(doc.nodes(), &mut path, &mut categories);
    }
    categories
}

fn traverse_nodes(nodes: &[KdlNode], path: &mut Vec<String>, categories: &mut HashSet<Category>) {
    for node in nodes {
        if node.name().value().starts_with("cat:") {
            let cat_name = node.name().value().trim_start_matches("cat:").to_string();

            categories.insert(Category {
                name: cat_name.clone(),
                path: path.clone(),
            });

            path.push(cat_name);

            if let Some(children) = node.children() {
                traverse_nodes(children.nodes(), path, categories);
            }

            path.pop();
        } else if let Some(children) = node.children() {
            traverse_nodes(children.nodes(), path, categories);
        }
    }
}

pub fn prompt_pkgs_ins(app: &App) -> Result<Vec<Package>> {
    let pkg_manager = &app.config.package_manager;

    let output = cmd!(pkg_manager, "-Qq")
        .pipe(cmd!(
            "fzf",
            "--multi",
            "--preview",
            format!("{} -Qi {{}}", pkg_manager),
            "--preview-window=right:75%",
            "--layout=default"
        ))
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to get installed packages: {}", e))?;

    Ok(output.lines().map(Package::from).collect())
}

pub fn prompt_pkgs_exp(app: &App) -> Result<Vec<Package>> {
    let pkg_manager = &app.config.package_manager;

    let output = cmd!(pkg_manager, "-Qqe")
        .pipe(cmd!(
            "fzf",
            "--multi",
            "--preview",
            format!("{} -Qi {{}}", pkg_manager),
            "--preview-window=right:75%",
            "--layout=default"
        ))
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to get installed packages: {}", e))?;

    Ok(output.lines().map(Package::from).collect())
}

pub fn prompt_pkgs_all(app: &App) -> Result<Vec<Package>> {
    let pkg_manager = &app.config.package_manager;

    let output = cmd!(pkg_manager, "-Slq")
        .pipe(cmd!(
            "fzf",
            "--multi",
            "--preview",
            format!("{} -Sii {{}}", pkg_manager),
            "--preview-window=right:75%",
            "--layout=default"
        ))
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to get installed packages: {}", e))?;

    Ok(output.lines().map(Package::from).collect())
}
