use std::{
    collections::HashMap,
    env, fs,
    path::Path,
};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let shader_dir = Path::new("src/shape/compiled/shader/template");

    // Re-run build if shader files change
    println!("cargo::rerun-if-changed={}", shader_dir.display());
    for entry in fs::read_dir(shader_dir).expect("Failed to read shader directory") {
        let entry = entry.unwrap();
        println!("cargo::rerun-if-changed={}", entry.path().display());
    }

    // Load all shader sources
    let mut modules: HashMap<String, String> = HashMap::new();

    for entry in fs::read_dir(shader_dir).expect("Failed to read shader directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "wgsl") {
            let content = fs::read_to_string(&path).expect("Failed to read shader file");

            // Extract module name from #define_import_path
            let module_name = content
                .lines()
                .find(|line| line.starts_with("#define_import_path"))
                .map(|line| line.trim_start_matches("#define_import_path").trim().to_string())
                .unwrap_or_else(|| {
                    path.file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                });

            modules.insert(module_name, content);
        }
    }

    // Compose the shader by processing imports
    let main_src = modules.get("template").expect("main.wgsl must define template module");
    let composed = compose_shader(main_src, &modules);

    // Validate with naga_oil (this will catch errors in the composed shader)
    use naga_oil::compose::{Composer, NagaModuleDescriptor};
    let mut composer = Composer::default();
    composer
        .make_naga_module(NagaModuleDescriptor {
            source: &composed,
            file_path: "composed.wgsl",
            ..Default::default()
        })
        .expect("Failed to validate composed shader");

    // Write composed WGSL to OUT_DIR
    let out_path = Path::new(&out_dir).join("shader_template.wgsl");
    fs::write(&out_path, &composed).expect("Failed to write shader module");

    println!(
        "cargo::warning=Composed shader template to {} ({} bytes)",
        out_path.display(),
        composed.len()
    );
}

fn compose_shader(source: &str, modules: &HashMap<String, String>) -> String {
    let mut output = String::new();
    let mut imported_modules: Vec<String> = Vec::new();

    // First, collect all imports and resolve them recursively
    collect_imports(source, modules, &mut imported_modules, &mut Vec::new());

    // Output imported modules first (in dependency order)
    for module_name in &imported_modules {
        if let Some(module_src) = modules.get(module_name) {
            output.push_str(&strip_directives(module_src));
            output.push_str("\n\n");
        }
    }

    // Output main module (without #import and #define_import_path)
    output.push_str(&strip_directives(source));

    output
}

fn collect_imports(
    source: &str,
    modules: &HashMap<String, String>,
    imported: &mut Vec<String>,
    stack: &mut Vec<String>,
) {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#import") {
            // Parse import: #import module::name or #import module::name::{items}
            let import_path = trimmed
                .trim_start_matches("#import")
                .trim()
                .split("::{")
                .next()
                .unwrap()
                .split("::")
                .take(2) // Take up to 2 components (e.g., template::constants)
                .collect::<Vec<_>>()
                .join("::");

            // Skip if already imported or in current stack (circular)
            if imported.contains(&import_path) || stack.contains(&import_path) {
                continue;
            }

            // Get the module and recursively process its imports
            if let Some(module_src) = modules.get(&import_path) {
                stack.push(import_path.clone());
                collect_imports(module_src, modules, imported, stack);
                stack.pop();
                imported.push(import_path);
            }
        }
    }
}

fn strip_directives(source: &str) -> String {
    let mut output = Vec::new();
    let mut in_braced_import = false;
    let mut brace_depth = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip #define_import_path lines
        if trimmed.starts_with("#define_import_path") {
            continue;
        }

        // Check if we're starting a new import
        if trimmed.starts_with("#import") {
            // Count braces on this line
            let open_braces = line.chars().filter(|&c| c == '{').count();
            let close_braces = line.chars().filter(|&c| c == '}').count();
            brace_depth = open_braces as i32 - close_braces as i32;

            // If braces are unbalanced, we're in a multi-line import
            if brace_depth > 0 {
                in_braced_import = true;
            }
            // Skip this import line regardless
            continue;
        }

        // If we're in a multi-line braced import, track braces
        if in_braced_import {
            for c in line.chars() {
                if c == '{' {
                    brace_depth += 1;
                } else if c == '}' {
                    brace_depth -= 1;
                }
            }

            // Import ends when all braces are closed
            if brace_depth <= 0 {
                in_braced_import = false;
            }
            continue;
        }

        output.push(line);
    }

    output.join("\n")
}
