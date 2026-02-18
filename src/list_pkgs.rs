use anyhow::{Context, Result, bail};
use kdl::KdlDocument;
use std::fs;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::pacman::run_pacman;

pub fn get_exp_pkg_list(log_file_path: &Path) -> Result<Vec<String>> {
    let output = run_pacman(&["-Qqe"])?;
    let mut explicit_pkgs: HashMap<String, Option<i32>> =
        output.lines().map(|s| (s.to_string(), None)).collect();

    let log_file = File::open(log_file_path)?; // TODO: log_file based sortering should be optional
    let reader = BufReader::new(log_file);

    let mut pkg_num = 0;
    for line in reader.lines() {
        let line = line?;
        if line.contains("[ALPM] installed") {
            // Parse the line: [timestamp] [ALPM] installed package_name (version)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                // let timestamp = &parts[0][1..parts[0].len() - 1]; // Remove the surrounding brackets
                let package_with_version = parts[3];

                // Extract just the package name (remove version in parentheses if present)
                let package_name = if let Some(start_idx) = package_with_version.find('(') {
                    package_with_version[..start_idx].trim_end()
                } else {
                    package_with_version
                };

                if !explicit_pkgs.contains_key(package_name) {
                    continue;
                }

                // Store only the first installation time found (like rg -m1)
                explicit_pkgs
                    .entry(package_name.to_string())
                    .and_modify(|existing| {
                        if existing.is_none() {
                            *existing = Some(pkg_num);
                        }
                    });
                pkg_num += 1;
            }
        }
    }
    let mut installation_timesvec: Vec<(String, Option<i32>)> = explicit_pkgs.into_iter().collect();
    installation_timesvec.sort_by(|a, b| a.1.cmp(&b.1));

    Ok(installation_timesvec.into_iter().map(|x| x.0).collect())
}

/// Returns a tuple of (installed_only, declared_only)
pub fn get_pkg_diff(
    documents: &Vec<(PathBuf, KdlDocument)>,
    pacman_log_path: &Path,
) -> Result<(Vec<String>, Vec<String>)> {
    let installed_pkgs = get_exp_pkg_list(pacman_log_path)?;
    let declared_pkgs = get_declared_pkg_list(documents)?;

    let declared_names: HashSet<String> =
        declared_pkgs.into_iter().map(strip_repo_prefix).collect();

    let installed_set: HashSet<String> = installed_pkgs.iter().cloned().collect();

    let installed_only: Vec<String> = installed_pkgs
        .into_iter()
        .filter(|pkg| !declared_names.contains(pkg))
        .collect();

    let declared_only: Vec<String> = declared_names
        .into_iter()
        .filter(|pkg| !installed_set.contains(pkg))
        .collect();

    Ok((installed_only, declared_only))
}

pub fn get_declared_pkg_list(documents: &Vec<(PathBuf, KdlDocument)>) -> Result<HashSet<String>> {
    let mut packages = HashSet::new();

    for (_, doc) in documents {
        collect_packages_from_doc(doc, &mut packages)?;
    }

    Ok(packages)
}

// Collect all KDL documents (main and included)
pub fn collect_documents(root_path: &Path) -> Result<Vec<(PathBuf, KdlDocument)>> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut documents = Vec::new();

    collect_documents_recursive(root_path, &mut visited, &mut documents)?;

    Ok(documents)
}

// Recursive helper for document collection
fn collect_documents_recursive(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    documents: &mut Vec<(PathBuf, KdlDocument)>,
) -> Result<()> {
    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize path: {}", path.display()))?;

    // Check for cycles
    if visited.contains(&canonical_path) {
        bail!("cyclic include detected: {}", canonical_path.display());
    }

    visited.insert(canonical_path.clone());

    // Parse the document
    let doc = parse_kdl_document(&canonical_path)?;

    // Process top-level @include nodes only
    let base_dir = canonical_path
        .parent()
        .context("config file must have a parent directory")?;

    for node in doc.nodes() {
        if node.name().value() == "@include" {
            // Handle include at top level
            if let Some(entry) = node.entries().first()
                && let Some(path_str) = entry.value().as_string()
            {
                let include_path = base_dir.join(path_str);
                collect_documents_recursive(&include_path, visited, documents)
                    .with_context(|| format!("failed to include file: {path_str}"))?;
            }
        }
        // Note: We don't process children for @include because includes should not be nested
    }

    documents.push((canonical_path.clone(), doc));
    Ok(())
}

// Parse a single KDL document from a file
fn parse_kdl_document(config_file_path: &Path) -> Result<KdlDocument> {
    let src = fs::read_to_string(config_file_path)
        .with_context(|| format!("failed to read config file: {}", config_file_path.display()))?;

    src.parse::<KdlDocument>()
        .map_err(|e| {
            let report = miette::Report::new(e);
            anyhow::anyhow!("Failed to parse KDL: {report:?}").context("KDL parsing failed")
        })
        .with_context(|| format!("failed to parse KDL from: {}", config_file_path.display()))
}

// Collect packages from a single document
fn collect_packages_from_doc(doc: &KdlDocument, packages: &mut HashSet<String>) -> Result<()> {
    for node in doc.nodes() {
        let node_name = node.name().value();

        match node_name {
            name if !name.contains([':', '@']) => {
                // Regular package - add it to the set
                packages.insert(name.to_string());

                // Recursively process children
                if let Some(children) = node.children() {
                    collect_packages_from_doc(children, packages)?;
                }
            }
            name if name.starts_with("cat:") => {
                // Category - just process children
                if let Some(children) = node.children() {
                    collect_packages_from_doc(children, packages)?;
                }
            }
            _ => {} // Ignore other nodes
        }
    }
    Ok(())
}
fn strip_repo_prefix(pkg: String) -> String {
    pkg.rsplit_once('/')
        .map(|(_, name)| name.to_string())
        .unwrap_or(pkg)
}
