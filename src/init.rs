use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

struct TemplateSpec {
    canonical_name: &'static str,
    description: &'static str,
    body: &'static str,
}

const RUST_TEMPLATE: &str = include_str!("../templates/rust.yaml");
const FRONTEND_TEMPLATE: &str = include_str!("../templates/frontend.yaml");
const SHADCN_TEMPLATE: &str = include_str!("../templates/shadcn-ui.yaml");

pub fn write_template(stack: &str, output: &Path, force: bool) -> Result<()> {
    let spec = resolve_template(stack)?;

    if output.exists() && !force {
        bail!(
            "{} already exists. Re-run with --force to overwrite it.",
            output.display()
        );
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(output, spec.body)
        .with_context(|| format!("Failed to write {}", output.display()))?;

    println!(
        "wrote {} template to {}",
        spec.canonical_name,
        output.display()
    );
    println!("{}", spec.description);
    Ok(())
}

fn resolve_template(stack: &str) -> Result<TemplateSpec> {
    let normalized = stack.trim().to_ascii_lowercase();

    let spec = match normalized.as_str() {
        "rust" => TemplateSpec {
            canonical_name: "rust",
            description: "Rust template with Context7 and GitHub MCP defaults.",
            body: RUST_TEMPLATE,
        },
        "frontend" | "ts" | "typescript" | "react" => TemplateSpec {
            canonical_name: "frontend",
            description:
                "Frontend template for TypeScript/React work with shadcn and browser MCPs.",
            body: FRONTEND_TEMPLATE,
        },
        "shadcn" | "shadcn-ui" | "shadcn/ui" => TemplateSpec {
            canonical_name: "shadcn-ui",
            description: "UI template focused on shadcn/ui projects.",
            body: SHADCN_TEMPLATE,
        },
        _ => bail!(
            "Unsupported stack `{stack}`. Try one of: rust, frontend, typescript, react, shadcn-ui."
        ),
    };

    Ok(spec)
}
