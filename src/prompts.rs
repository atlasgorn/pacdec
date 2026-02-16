use anyhow::Result;
use duct::cmd;
use inquire::Select;

pub fn prompt_category(categories: Vec<String>) -> Result<String> {
    match Select::new("input category", categories).prompt() {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(e.into()),
    }
}

pub fn prompt_pkgs_ins() -> Result<Vec<String>> {
    let pkg_manager = "paru"; // TODO: different pacmans

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

pub fn prompt_pkgs_exp() -> Result<Vec<String>> {
    let pkg_manager = "paru"; // TODO: different pacmans

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

pub fn prompt_pkgs_all() -> Result<Vec<String>> {
    let pkg_manager = "paru"; // TODO: different pacmans

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
