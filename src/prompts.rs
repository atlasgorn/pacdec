use anyhow::Result;
use duct::cmd;
use inquire::Select;
use std::collections::HashSet;

use crate::app::App;

pub fn prompt_category(app: &App) -> Result<String> {
    let mut categories: Vec<String> = collect_categories(app).into_iter().collect();
    let default_cat = &app.config.default_category;
    categories.sort();
    if categories.contains(default_cat) {
        categories.retain(|cat| cat != default_cat);
        categories.insert(0, default_cat.clone());
    }
    match Select::new("input category", categories).prompt() {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(e.into()),
    }
}

pub fn collect_categories(app: &App) -> HashSet<String> {
    let mut categories = HashSet::new();

    for (_, doc) in &app.docs {
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

pub fn prompt_pkgs_ins(app: &App) -> Result<Vec<String>> {
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

    Ok(output.lines().map(String::from).collect())
}

pub fn prompt_pkgs_exp(app: &App) -> Result<Vec<String>> {
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

    Ok(output.lines().map(String::from).collect())
}

pub fn prompt_pkgs_all(app: &App) -> Result<Vec<String>> {
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

    Ok(output.lines().map(String::from).collect())
}
