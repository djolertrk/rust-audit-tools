use cargo_metadata::MetadataCommand;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use syn::visit::Visit;
use syn::spanned::Spanned;

use proc_macro2::Span;

// A simple representation of one call site.
#[derive(Debug)]
struct CallDetail {
    callee: String,
    filename: String,
    line: usize,
    column: usize,
}

type CallGraphMap = HashMap<String, Vec<CallDetail>>;

struct CallGraphCollector<'a> {
    filename: &'a str,
    current_fn: Option<String>,
    call_graph: &'a mut CallGraphMap,
    src_lines: Vec<&'a str>,
}

impl<'a> CallGraphCollector<'a> {
    fn new(filename: &'a str, src_content: &'a str, call_graph: &'a mut CallGraphMap) -> Self {
        let src_lines = src_content.lines().collect();
        CallGraphCollector {
            filename,
            current_fn: None,
            call_graph,
            src_lines,
        }
    }

    fn offset_to_line_column(&self, offset: usize) -> (usize, usize) {
        // If you still want a naive fallback approach, you can keep this.
        let mut remaining = offset;
        for (line_index, line) in self.src_lines.iter().enumerate() {
            if remaining <= line.len() {
                return (line_index + 1, remaining + 1);
            } else {
                remaining -= line.len() + 1;
            }
        }
        (0, 0)
    }
}

impl<'ast, 'a> Visit<'ast> for CallGraphCollector<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let fn_name = node.sig.ident.to_string();
        self.current_fn = Some(fn_name.clone());
        self.call_graph.entry(fn_name).or_default();

        syn::visit::visit_item_fn(self, node);
        self.current_fn = None;
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            let callee_str = expr_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if let Some(ref caller) = self.current_fn {
                let span = node.func.span();
                let start = span.start();
                let line = start.line + 1;
                let column = start.column + 1;

                let calls = self.call_graph.entry(caller.clone()).or_default();
                calls.push(CallDetail {
                    callee: callee_str,
                    filename: self.filename.to_string(),
                    line,
                    column,
                });
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let crate_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir()?
    };

    let mut cmd = MetadataCommand::new();
    cmd.manifest_path(
        crate_path
            .join("Cargo.toml")
            .to_string_lossy()
            .to_string(),
    );
    let metadata = cmd.exec()?;

    let package = metadata
        .packages
        .get(0)
        .ok_or("No package found in Cargo.toml")?;

    let mut rs_files = Vec::new();
    for target in &package.targets {
        rs_files.push(&target.src_path);
    }

    let mut call_graph: CallGraphMap = HashMap::new();

    for rs_path in rs_files {
        let file_content = fs::read_to_string(rs_path)?;
        let syntax = syn::parse_file(&file_content)?;

        let mut visitor = CallGraphCollector::new(rs_path.as_str(), &file_content, &mut call_graph);
        visitor.visit_file(&syntax);
    }

    // Output a .dot graph
    println!("digraph G {{");
    for (caller, calls) in &call_graph {
        for c in calls {
            let label = if c.line > 0 {
                format!("{}:{}:{}", c.filename, c.line, c.column)
            } else {
                c.filename.clone()
            };

            println!(
                r#"    "{caller}" -> "{callee}" [label="{label}"];"#,
                caller = caller,
                callee = c.callee,
                label = label
            );
        }
    }
    println!("}}");

    Ok(())
}
