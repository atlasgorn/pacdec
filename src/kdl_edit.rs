use anyhow::Result;
use anyhow::bail;
use colored::*;
use kdl::{FormatConfig, KdlDocument, KdlNode};

pub fn add_pkgs(doc: &mut KdlDocument, category: &str, pkgs: &[&str]) -> Result<()> {
    let category = format!("cat:{category}");
    let mut stack = vec![doc.nodes_mut()];
    let mut cat_count = 0;

    while let Some(nodes) = stack.pop() {
        for node in nodes {
            if node.name().value() == category {
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
                    .extend(pkgs.iter().map(|&pkg| {
                        let mut new_node = KdlNode::new(pkg);
                        new_node.autoformat_config(
                            &FormatConfig::builder().indent_level(indent + 1).build(),
                        );
                        new_node
                    }));
            }
            if let Some(children) = node.children_mut() {
                stack.push(children.nodes_mut());
            }
        }
    }
    if cat_count > 1 {
        println!("{}", format!(
            "category {category} exists in several places, adding package(s) only to 1st occurrence") // TODO: prompt to choose occurrence 
            .yellow().bold()
        );
    } else if cat_count == 0 {
        bail!("no such category \"{category}\" exists");
    }

    Ok(())
}

pub fn remove_pkgs(doc: &mut KdlDocument, pkgs: &[impl AsRef<str>]) -> Result<()> {
    let mut stack = vec![doc.nodes_mut()];
    let comment = true;

    while let Some(nodes) = stack.pop() {
        if !comment {
            nodes.retain(|node| !pkgs.iter().any(|pkg| pkg.as_ref() == node.name().value()));
        }
        for node in nodes {
            if comment {
                if pkgs.iter().any(|pkg| pkg.as_ref() == node.name().value()) {
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
