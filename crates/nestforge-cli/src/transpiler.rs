use anyhow::{bail, Result};
use regex::Regex;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub struct Transpiler {
    source_dir: PathBuf,
    cache_dir: PathBuf,
    import_regex: Regex,
    module_import_regex: Regex,
}

impl Transpiler {
    pub fn new(source_dir: &Path, cache_dir: &Path) -> Result<Self> {
        let import_regex = Regex::new(r#"import\s*\{([^}]+)\}\s*from\s*["']([^"']+)["']\s*;"#)?;

        let module_import_regex =
            Regex::new(r#"import\s+([A-Za-z0-9_]+)\s+from\s+["']([^"']+)["']\s*;"#)?;

        Ok(Self {
            source_dir: source_dir.to_path_buf(),
            cache_dir: cache_dir.to_path_buf(),
            import_regex,
            module_import_regex,
        })
    }

    pub fn run(&self) -> Result<()> {
        if !self.source_dir.exists() {
            bail!("Source directory does not exist: {:?}", self.source_dir);
        }

        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        fs::create_dir_all(&self.cache_dir)?;

        self.process_directory(&self.source_dir, &self.cache_dir, "")?;

        Ok(())
    }

    fn process_directory(&self, source_dir: &Path, cache_dir: &Path, prefix: &str) -> Result<()> {
        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir)?;
        }

        let mut module_items: Vec<String> = Vec::new();

        for entry in fs::read_dir(source_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if path.is_dir() {
                if file_name.starts_with('.') || file_name == "target" {
                    continue;
                }

                let module_name = self.to_snake_case(file_name);
                let source_subdir = source_dir.join(file_name);
                let cache_subdir = cache_dir.join(&module_name);

                self.process_directory(&source_subdir, &cache_subdir, &module_name)?;

                module_items.push(module_name);
            } else if path.is_file() {
                if file_name.starts_with('.') {
                    continue;
                }

                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        let transpiled = self.transpile_file(&path, prefix)?;
                        if !transpiled.is_empty() {
                            let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");

                            let output_path = cache_dir.join(format!("{}.rs", stem));
                            fs::write(&output_path, &transpiled)?;

                            module_items.push(stem.to_string());
                        }
                    }
                }
            }
        }

        if !module_items.is_empty() {
            let mod_content = self.generate_mod_file(&module_items);
            let mod_path = cache_dir.join("mod.rs");
            fs::write(&mod_path, mod_content)?;
        }

        Ok(())
    }

    fn transpile_file(&self, path: &Path, prefix: &str) -> Result<String> {
        let content = fs::read_to_string(path)?;

        if !content.contains("import ") {
            return Ok(content);
        }

        let mut result = content.clone();
        let mut imports_map: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for cap in self.import_regex.captures_iter(&content) {
            let symbols = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let source = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let transformed_source = self.transform_path(source, prefix);
            let module_path = transformed_source.replace("::", "/");

            for symbol in symbols.split(',') {
                let symbol = symbol.trim();
                if !symbol.is_empty() {
                    imports_map
                        .entry(module_path.clone())
                        .or_default()
                        .push((symbol.to_string(), symbol.to_string()));
                }
            }

            let import_pattern = format!(
                r#"import\s*\{{}}\s*from\s*["']{}["']\s*;"#,
                regex::escape(source)
            );
            result = Regex::new(&import_pattern)?
                .replace_all(&result, "")
                .to_string();
        }

        for cap in self.module_import_regex.captures_iter(&content) {
            let symbol = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let source = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let transformed_source = self.transform_path(source, prefix);

            imports_map
                .entry(transformed_source)
                .or_default()
                .push((symbol.to_string(), symbol.to_string()));

            let import_pattern = format!(
                r#"import\s+{}\s+from\s*["']{}["']\s*;"#,
                symbol,
                regex::escape(source)
            );
            result = Regex::new(&import_pattern)?
                .replace_all(&result, "")
                .to_string();
        }

        let mut use_statements: Vec<String> = Vec::new();
        for (module, symbols) in imports_map {
            if symbols.is_empty() {
                continue;
            }

            let symbol_list: Vec<String> = symbols.iter().map(|(orig, _)| orig.clone()).collect();

            let use_stmt = if symbol_list.len() == 1 {
                format!("use {}::{};", module, symbol_list[0])
            } else {
                format!("use {}::{{{}}};", module, symbol_list.join(", "))
            };
            use_statements.push(use_stmt);
        }

        if !use_statements.is_empty() {
            let use_block = use_statements.join("\n");
            result = format!("{}\n\n{}", use_block, result);
        }

        Ok(result)
    }

    fn transform_path(&self, source: &str, _prefix: &str) -> String {
        if source.starts_with("nestforge/") {
            let module = source.strip_prefix("nestforge/").unwrap_or(source);
            format!("nestforge::{}", module.replace('/', "::"))
        } else if source.starts_with('@') {
            source.replace('@', "nestforge::")
        } else if source.starts_with("./") || source.starts_with("../") {
            let clean_path = source.trim_start_matches("./").trim_start_matches("../");
            let path_parts: Vec<&str> = clean_path.split('/').collect();

            let rust_path: String = path_parts
                .iter()
                .map(|p| self.to_snake_case(p))
                .collect::<Vec<_>>()
                .join("::");

            if source.starts_with("./") {
                format!("self::{}", rust_path)
            } else {
                let parent_count = source.matches("..").count();
                let super_prefix = (0..parent_count)
                    .map(|_| "super")
                    .collect::<Vec<_>>()
                    .join("::");

                if rust_path.is_empty() {
                    super_prefix
                } else if super_prefix.is_empty() {
                    rust_path
                } else {
                    format!("{}::{}", super_prefix, rust_path)
                }
            }
        } else {
            source.replace('-', "_").replace('/', "::")
        }
    }

    pub fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();

        for word in s.split(|c: char| !c.is_alphanumeric()) {
            if word.is_empty() {
                continue;
            }
            if !result.is_empty() {
                result.push('_');
            }
            result.push_str(&word.to_lowercase());
        }

        if result.is_empty() {
            s.to_lowercase()
        } else {
            result
        }
    }

    fn generate_mod_file(&self, modules: &[String]) -> String {
        let mut content = String::new();

        for module in modules {
            content.push_str(&format!("pub mod {};\n", module));
        }

        content
    }
}

pub fn transpile_project(source_dir: &Path, cache_dir: &Path) -> Result<PathBuf> {
    let transpiler = Transpiler::new(source_dir, cache_dir)?;
    transpiler.run()?;
    Ok(cache_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_transform_path_nestforge_module() {
        let temp_dir = TempDir::new().unwrap();
        let transpiler = Transpiler::new(temp_dir.path(), temp_dir.path()).unwrap();

        let result = transpiler.transform_path("nestforge/common", "");
        assert_eq!(result, "nestforge::common");

        let result = transpiler.transform_path("nestforge/http", "");
        assert_eq!(result, "nestforge::http");
    }

    #[test]
    fn test_transform_path_relative_import() {
        let temp_dir = TempDir::new().unwrap();
        let transpiler = Transpiler::new(temp_dir.path(), temp_dir.path()).unwrap();

        let result = transpiler.transform_path("./users.service", "users");
        assert!(result.contains("users_service"));
    }

    #[test]
    fn test_transform_path_parent_import() {
        let temp_dir = TempDir::new().unwrap();
        let transpiler = Transpiler::new(temp_dir.path(), temp_dir.path()).unwrap();

        let result = transpiler.transform_path("../config", "users");
        assert!(result.contains("super"));
    }

    #[test]
    fn test_to_snake_case() {
        let temp_dir = TempDir::new().unwrap();
        let transpiler = Transpiler::new(temp_dir.path(), temp_dir.path()).unwrap();

        assert_eq!(transpiler.to_snake_case("users_service"), "users_service");
        assert_eq!(
            transpiler.to_snake_case("auth_controller"),
            "auth_controller"
        );
        assert_eq!(transpiler.to_snake_case("my_controller"), "my_controller");
        assert_eq!(transpiler.to_snake_case("users"), "users");
    }

    #[test]
    fn test_generate_mod_file() {
        let temp_dir = TempDir::new().unwrap();
        let transpiler = Transpiler::new(temp_dir.path(), temp_dir.path()).unwrap();

        let modules = vec!["users".to_string(), "controllers".to_string()];
        let result = transpiler.generate_mod_file(&modules);

        assert!(result.contains("pub mod users;"));
        assert!(result.contains("pub mod controllers;"));
    }
}
