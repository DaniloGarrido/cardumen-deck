use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use toml::Value;

const MAX_DEPENDENCIES: usize = 200;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDependency {
    pub name: String,
    pub version: Option<String>,
    pub kind: String,
    pub ecosystem: String,
    pub manifest: String,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDependencyReport {
    pub dependencies: Vec<ServiceDependency>,
    pub manifests: Vec<String>,
    pub framework: Option<String>,
    pub total: usize,
    pub truncated: bool,
    pub warnings: Vec<String>,
}

#[derive(Default)]
struct Accumulator {
    dependencies: Vec<ServiceDependency>,
    index: HashMap<String, usize>,
    manifests: Vec<String>,
    warnings: Vec<String>,
    framework: Option<String>,
}

impl Accumulator {
    fn manifest(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.manifests.contains(&name) {
            self.manifests.push(name);
        }
    }

    fn warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    fn add(&mut self, dependency: ServiceDependency) {
        let key = format!(
            "{}:{}",
            dependency.ecosystem,
            dependency.name.to_lowercase()
        );

        if let Some(index) = self.index.get(&key).copied() {
            let existing = &mut self.dependencies[index];
            if kind_priority(&dependency.kind) > kind_priority(&existing.kind) {
                *existing = dependency;
            } else if existing.version.is_none() && dependency.version.is_some() {
                existing.version = dependency.version;
            }
            return;
        }

        self.index.insert(key, self.dependencies.len());
        self.dependencies.push(dependency);
    }

    fn into_report(mut self) -> ServiceDependencyReport {
        let total = self.dependencies.len();
        self.dependencies.sort_by(|left, right| {
            kind_priority(&right.kind)
                .cmp(&kind_priority(&left.kind))
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });

        let truncated = total > MAX_DEPENDENCIES;
        self.dependencies.truncate(MAX_DEPENDENCIES);

        ServiceDependencyReport {
            dependencies: self.dependencies,
            manifests: self.manifests,
            framework: self.framework,
            total,
            truncated,
            warnings: self.warnings,
        }
    }
}

fn kind_priority(kind: &str) -> u8 {
    match kind {
        "runtime" => 6,
        "peer" => 5,
        "optional" => 4,
        "build" => 3,
        "dev" => 2,
        "indirect" => 1,
        _ => 0,
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Result<String, String> {
    if bytes.len() % 2 != 0 {
        return Err("cantidad impar de bytes UTF-16".to_string());
    }

    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| {
            if little_endian {
                u16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], chunk[1]])
            }
        })
        .collect();

    String::from_utf16(&units).map_err(|error| error.to_string())
}

fn decode_text(bytes: &[u8]) -> Result<(String, Option<String>), String> {
    if let Some(content) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return decode_utf16(content, true).map(|text| (text, None));
    }
    if let Some(content) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return decode_utf16(content, false).map(|text| (text, None));
    }

    let content = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(content) {
        Ok(text) => Ok((text.to_string(), None)),
        Err(_) => Ok((
            String::from_utf8_lossy(content).into_owned(),
            Some("El archivo no está en UTF-8; se leyó con reemplazo de caracteres.".to_string()),
        )),
    }
}

fn read_text(path: &Path, acc: &mut Accumulator) -> Option<String> {
    match fs::read(path) {
        Ok(bytes) => match decode_text(&bytes) {
            Ok((content, warning)) => {
                if let Some(warning) = warning {
                    acc.warning(format!("{}: {warning}", path.to_string_lossy()));
                }
                Some(content)
            }
            Err(error) => {
                acc.warning(format!(
                    "No se pudo decodificar {}: {error}",
                    path.to_string_lossy()
                ));
                None
            }
        },
        Err(error) => {
            acc.warning(format!(
                "No se pudo leer {}: {error}",
                path.to_string_lossy()
            ));
            None
        }
    }
}

pub fn detect(cwd: &str) -> Result<ServiceDependencyReport, String> {
    let mut acc = Accumulator::default();
    let trimmed = cwd.trim();

    if trimmed.is_empty() {
        acc.warning("El servicio no tiene una carpeta de trabajo configurada.");
        return Ok(acc.into_report());
    }

    let path = PathBuf::from(trimmed);
    if !path.is_dir() {
        acc.warning(format!(
            "La carpeta de trabajo no existe: {}",
            path.to_string_lossy()
        ));
        return Ok(acc.into_report());
    }

    let root = fs::canonicalize(&path).unwrap_or(path);

    parse_package_json(&root, &mut acc);
    parse_maven(&root, &mut acc);
    parse_gradle(&root, &mut acc);
    parse_python(&root, &mut acc);
    parse_cargo(&root, &mut acc);
    parse_go(&root, &mut acc);
    parse_dotnet(&root, &mut acc);

    Ok(acc.into_report())
}

fn parse_package_json(root: &Path, acc: &mut Accumulator) {
    let path = root.join("package.json");
    if !path.is_file() {
        return;
    }

    acc.manifest("package.json");
    let Some(content) = read_text(&path, acc) else {
        return;
    };
    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            acc.warning(format!("package.json inválido: {error}"));
            return;
        }
    };

    for (field, kind) in [
        ("dependencies", "runtime"),
        ("devDependencies", "dev"),
        ("peerDependencies", "peer"),
        ("optionalDependencies", "optional"),
    ] {
        let Some(dependencies) = parsed.get(field).and_then(|value| value.as_object()) else {
            continue;
        };

        for (name, value) in dependencies {
            acc.add(ServiceDependency {
                name: name.clone(),
                version: value.as_str().map(str::to_string),
                kind: kind.to_string(),
                ecosystem: "node".to_string(),
                manifest: "package.json".to_string(),
                group: None,
            });
        }
    }
}

fn xml_child_text(node: roxmltree::Node, name: &str) -> Option<String> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == name)
        .and_then(|child| child.text())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn parse_maven(root: &Path, acc: &mut Accumulator) {
    let path = root.join("pom.xml");
    if !path.is_file() {
        return;
    }

    acc.manifest("pom.xml");
    let Some(content) = read_text(&path, acc) else {
        return;
    };
    let document = match roxmltree::Document::parse(&content) {
        Ok(document) => document,
        Err(error) => {
            acc.warning(format!("pom.xml inválido: {error}"));
            return;
        }
    };

    for dependencies in document
        .root_element()
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "dependencies")
    {
        for dependency in dependencies
            .children()
            .filter(|node| node.is_element() && node.tag_name().name() == "dependency")
        {
            let Some(artifact) = xml_child_text(dependency, "artifactId") else {
                continue;
            };
            let group = xml_child_text(dependency, "groupId");
            let version = xml_child_text(dependency, "version");
            let scope = xml_child_text(dependency, "scope");
            let optional = xml_child_text(dependency, "optional")
                .map(|value| value.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

            let kind = if optional {
                "optional"
            } else {
                match scope.as_deref() {
                    Some("test") => "dev",
                    Some("provided") | Some("system") => "build",
                    _ => "runtime",
                }
            };

            acc.add(ServiceDependency {
                name: artifact,
                version,
                kind: kind.to_string(),
                ecosystem: "maven".to_string(),
                manifest: "pom.xml".to_string(),
                group,
            });
        }
    }
}

fn extract_quoted(value: &str) -> Option<&str> {
    for quote in ['\'', '"'] {
        if let Some(start) = value.find(quote) {
            let rest = &value[start + 1..];
            if let Some(end) = rest.find(quote) {
                return Some(&rest[..end]);
            }
        }
    }
    None
}

fn parse_gradle_file(path: &Path, manifest: &str, acc: &mut Accumulator) {
    let Some(content) = read_text(path, acc) else {
        return;
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }

        let Some((configuration, rest)) =
            trimmed.split_once(|character: char| character.is_whitespace() || character == '(')
        else {
            continue;
        };

        let kind = match configuration {
            "testImplementation" | "testRuntimeOnly" | "testCompileOnly" => Some("dev"),
            "compileOnly" | "annotationProcessor" | "kapt" => Some("build"),
            "implementation" | "api" | "runtimeOnly" | "developmentOnly" => Some("runtime"),
            _ => None,
        };
        let Some(kind) = kind else {
            continue;
        };

        let Some(coordinate) = extract_quoted(rest) else {
            continue;
        };
        if coordinate.starts_with(':') || coordinate.contains("project(") {
            continue;
        }

        let parts: Vec<&str> = coordinate.split(':').collect();
        let Some(artifact) = parts.get(1).copied().filter(|value| !value.is_empty()) else {
            continue;
        };

        acc.add(ServiceDependency {
            name: artifact.to_string(),
            version: parts.get(2).map(|value| value.to_string()),
            kind: kind.to_string(),
            ecosystem: "gradle".to_string(),
            manifest: manifest.to_string(),
            group: parts.first().map(|value| value.to_string()),
        });
    }
}

fn parse_gradle(root: &Path, acc: &mut Accumulator) {
    for manifest in ["build.gradle", "build.gradle.kts"] {
        let path = root.join(manifest);
        if !path.is_file() {
            continue;
        }
        acc.manifest(manifest);
        parse_gradle_file(&path, manifest, acc);
    }
}

fn python_requirement_kind(path: &Path) -> &'static str {
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name.contains("dev") || name.contains("test") || name.contains("local") {
        "dev"
    } else {
        "runtime"
    }
}

fn clean_python_version(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(
        trimmed
            .strip_prefix("===")
            .or_else(|| trimmed.strip_prefix("=="))
            .unwrap_or(trimmed)
            .trim()
            .to_string(),
    )
    .filter(|value| !value.is_empty())
}

fn add_python_requirement(raw: &str, kind: &str, manifest: &str, acc: &mut Accumulator) {
    let requirement = raw
        .split(';')
        .next()
        .unwrap_or(raw)
        .split(" #")
        .next()
        .unwrap_or(raw)
        .trim();

    if requirement.is_empty() || requirement.starts_with('-') {
        return;
    }
    if requirement.starts_with("git+")
        || requirement.starts_with("http://")
        || requirement.starts_with("https://")
    {
        acc.warning(format!(
            "Dependencia remota no listada en {manifest}: {requirement}"
        ));
        return;
    }

    let name_end = requirement
        .find(|character: char| {
            !(character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                || character == '.')
        })
        .unwrap_or(requirement.len());
    let name = &requirement[..name_end];
    if name.is_empty() {
        return;
    }

    let mut rest = &requirement[name_end..];
    if rest.starts_with('[') {
        if let Some(end) = rest.find(']') {
            rest = &rest[end + 1..];
        }
    }

    acc.add(ServiceDependency {
        name: name.to_string(),
        version: clean_python_version(rest),
        kind: kind.to_string(),
        ecosystem: "python".to_string(),
        manifest: manifest.to_string(),
        group: None,
    });
}

fn parse_requirements_file(
    root: &Path,
    path: &Path,
    acc: &mut Accumulator,
    visited: &mut HashSet<PathBuf>,
) {
    let canonical = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => return,
    };
    if !canonical.starts_with(root) {
        acc.warning(format!(
            "Se ignoró un requirements fuera del proyecto: {}",
            path.to_string_lossy()
        ));
        return;
    }
    if !visited.insert(canonical.clone()) {
        return;
    }

    let manifest = relative_path(root, &canonical);
    acc.manifest(&manifest);
    let Some(content) = read_text(&canonical, acc) else {
        return;
    };
    let kind = python_requirement_kind(&canonical);

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let include = trimmed
            .strip_prefix("-r ")
            .or_else(|| trimmed.strip_prefix("--requirement "))
            .map(str::trim);
        if let Some(include) = include {
            let target = canonical
                .parent()
                .unwrap_or(root)
                .join(include.trim_matches('"').trim_matches('\''));
            parse_requirements_file(root, &target, acc, visited);
            continue;
        }

        if trimmed.starts_with("-c ")
            || trimmed.starts_with("--constraint ")
            || trimmed.starts_with("--index-url")
            || trimmed.starts_with("--extra-index-url")
            || trimmed.starts_with("--find-links")
            || trimmed.starts_with("--trusted-host")
            || trimmed.starts_with("-e ")
            || trimmed.starts_with("--editable ")
        {
            continue;
        }

        add_python_requirement(trimmed, kind, &manifest, acc);
    }
}

fn collect_requirement_files(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_requirement_files(&path, depth - 1, files);
            continue;
        }
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if name.ends_with(".txt") && !name.starts_with("constraints") {
            files.push(path);
        }
    }
}

fn toml_version(value: &Value) -> Option<String> {
    match value {
        Value::String(version) => Some(version.clone()),
        Value::Table(table) => table
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                table
                    .get("workspace")
                    .and_then(Value::as_bool)
                    .filter(|value| *value)
                    .map(|_| "workspace".to_string())
            }),
        _ => None,
    }
}

fn parse_pyproject(root: &Path, acc: &mut Accumulator) {
    let path = root.join("pyproject.toml");
    if !path.is_file() {
        return;
    }

    acc.manifest("pyproject.toml");
    let Some(content) = read_text(&path, acc) else {
        return;
    };
    let document: Value = match toml::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            acc.warning(format!("pyproject.toml inválido: {error}"));
            return;
        }
    };

    if let Some(project) = document.get("project") {
        if let Some(dependencies) = project.get("dependencies").and_then(Value::as_array) {
            for dependency in dependencies.iter().filter_map(Value::as_str) {
                add_python_requirement(dependency, "runtime", "pyproject.toml", acc);
            }
        }

        if let Some(optional) = project
            .get("optional-dependencies")
            .and_then(Value::as_table)
        {
            for (group, dependencies) in optional {
                let kind = if group.to_lowercase().contains("dev") {
                    "dev"
                } else {
                    "optional"
                };
                if let Some(dependencies) = dependencies.as_array() {
                    for dependency in dependencies.iter().filter_map(Value::as_str) {
                        add_python_requirement(dependency, kind, "pyproject.toml", acc);
                    }
                }
            }
        }
    }

    let poetry = document.get("tool").and_then(|value| value.get("poetry"));

    if let Some(dependencies) = poetry
        .and_then(|value| value.get("dependencies"))
        .and_then(Value::as_table)
    {
        for (name, value) in dependencies {
            if name == "python" {
                continue;
            }
            let optional = value
                .as_table()
                .and_then(|table| table.get("optional"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            acc.add(ServiceDependency {
                name: name.clone(),
                version: toml_version(value),
                kind: if optional { "optional" } else { "runtime" }.to_string(),
                ecosystem: "python".to_string(),
                manifest: "pyproject.toml".to_string(),
                group: None,
            });
        }
    }

    if let Some(groups) = poetry
        .and_then(|value| value.get("group"))
        .and_then(Value::as_table)
    {
        for (group, value) in groups {
            let kind = if group.to_lowercase().contains("dev") {
                "dev"
            } else {
                "optional"
            };
            if let Some(dependencies) = value.get("dependencies").and_then(Value::as_table) {
                for (name, version) in dependencies {
                    acc.add(ServiceDependency {
                        name: name.clone(),
                        version: toml_version(version),
                        kind: kind.to_string(),
                        ecosystem: "python".to_string(),
                        manifest: "pyproject.toml".to_string(),
                        group: Some(group.clone()),
                    });
                }
            }
        }
    }

    if let Some(dependencies) = poetry
        .and_then(|value| value.get("dev-dependencies"))
        .and_then(Value::as_table)
    {
        for (name, version) in dependencies {
            acc.add(ServiceDependency {
                name: name.clone(),
                version: toml_version(version),
                kind: "dev".to_string(),
                ecosystem: "python".to_string(),
                manifest: "pyproject.toml".to_string(),
                group: None,
            });
        }
    }
}

fn parse_pipfile(root: &Path, acc: &mut Accumulator) {
    let path = root.join("Pipfile");
    if !path.is_file() {
        return;
    }

    acc.manifest("Pipfile");
    let Some(content) = read_text(&path, acc) else {
        return;
    };
    let document: Value = match toml::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            acc.warning(format!("Pipfile inválido: {error}"));
            return;
        }
    };

    for (section, kind) in [("packages", "runtime"), ("dev-packages", "dev")] {
        let Some(dependencies) = document.get(section).and_then(Value::as_table) else {
            continue;
        };
        for (name, value) in dependencies {
            acc.add(ServiceDependency {
                name: name.clone(),
                version: toml_version(value),
                kind: kind.to_string(),
                ecosystem: "python".to_string(),
                manifest: "Pipfile".to_string(),
                group: None,
            });
        }
    }
}

fn parse_python(root: &Path, acc: &mut Accumulator) {
    let is_django = root.join("manage.py").is_file()
        || root.join("settings.py").is_file()
        || acc.dependencies.iter().any(|dependency| {
            dependency.ecosystem == "python" && dependency.name.eq_ignore_ascii_case("django")
        });
    if is_django {
        acc.framework = Some("Django".to_string());
    }

    let mut files = Vec::new();
    let main_requirements = root.join("requirements.txt");
    if main_requirements.is_file() {
        files.push(main_requirements);
    }

    let requirements_dir = root.join("requirements");
    if requirements_dir.is_dir() {
        collect_requirement_files(&requirements_dir, 2, &mut files);
    }

    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|value| value.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if path.is_file()
                && name.starts_with("requirements")
                && name.ends_with(".txt")
                && !name.starts_with("constraints")
            {
                files.push(path);
            }
        }
    }

    let mut visited = HashSet::new();
    for file in files {
        parse_requirements_file(root, &file, acc, &mut visited);
    }

    parse_pyproject(root, acc);
    parse_pipfile(root, acc);

    if acc.framework.is_none()
        && acc.dependencies.iter().any(|dependency| {
            dependency.ecosystem == "python" && dependency.name.eq_ignore_ascii_case("django")
        })
    {
        acc.framework = Some("Django".to_string());
    }
}

fn parse_cargo(root: &Path, acc: &mut Accumulator) {
    let path = root.join("Cargo.toml");
    if !path.is_file() {
        return;
    }

    acc.manifest("Cargo.toml");
    let Some(content) = read_text(&path, acc) else {
        return;
    };
    let document: Value = match toml::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            acc.warning(format!("Cargo.toml inválido: {error}"));
            return;
        }
    };

    for (section, kind) in [
        ("dependencies", "runtime"),
        ("dev-dependencies", "dev"),
        ("build-dependencies", "build"),
    ] {
        let Some(dependencies) = document.get(section).and_then(Value::as_table) else {
            continue;
        };

        for (name, value) in dependencies {
            let internal_name = value
                .as_table()
                .and_then(|table| table.get("package"))
                .and_then(Value::as_str)
                .unwrap_or(name);
            let optional = value
                .as_table()
                .and_then(|table| table.get("optional"))
                .and_then(Value::as_bool)
                .unwrap_or(false);

            acc.add(ServiceDependency {
                name: internal_name.to_string(),
                version: toml_version(value),
                kind: if optional { "optional" } else { kind }.to_string(),
                ecosystem: "cargo".to_string(),
                manifest: "Cargo.toml".to_string(),
                group: None,
            });
        }
    }
}

fn parse_go(root: &Path, acc: &mut Accumulator) {
    let path = root.join("go.mod");
    if !path.is_file() {
        return;
    }

    acc.manifest("go.mod");
    let Some(content) = read_text(&path, acc) else {
        return;
    };

    let mut in_require = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("require (") {
            in_require = true;
            continue;
        }
        if in_require && trimmed == ")" {
            in_require = false;
            continue;
        }

        let require = if in_require {
            Some(trimmed)
        } else {
            trimmed.strip_prefix("require ").map(str::trim)
        };
        let Some(require) = require else {
            continue;
        };
        if require.is_empty() || require.starts_with("//") {
            continue;
        }

        let indirect = require.contains("// indirect");
        let without_comment = require.split("//").next().unwrap_or(require).trim();
        let mut parts = without_comment.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let version = parts.next().map(str::to_string);

        acc.add(ServiceDependency {
            name: name.to_string(),
            version,
            kind: if indirect { "indirect" } else { "runtime" }.to_string(),
            ecosystem: "go".to_string(),
            manifest: "go.mod".to_string(),
            group: None,
        });
    }
}

fn parse_dotnet(root: &Path, acc: &mut Accumulator) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if !path.is_file()
            || path
                .extension()
                .map(|extension| extension.to_string_lossy().to_lowercase())
                .as_deref()
                != Some("csproj")
        {
            continue;
        }

        let manifest = relative_path(root, &path);
        acc.manifest(&manifest);
        let Some(content) = read_text(&path, acc) else {
            continue;
        };
        let document = match roxmltree::Document::parse(&content) {
            Ok(document) => document,
            Err(error) => {
                acc.warning(format!("{manifest} inválido: {error}"));
                continue;
            }
        };

        for reference in document
            .descendants()
            .filter(|node| node.is_element() && node.tag_name().name() == "PackageReference")
        {
            let Some(name) = reference
                .attribute("Include")
                .or_else(|| reference.attribute("Update"))
            else {
                continue;
            };
            let version = reference
                .attribute("Version")
                .map(str::to_string)
                .or_else(|| xml_child_text(reference, "Version"));

            acc.add(ServiceDependency {
                name: name.to_string(),
                version,
                kind: "runtime".to_string(),
                ecosystem: "dotnet".to_string(),
                manifest: manifest.clone(),
                group: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "cardumen-deck-dependencies-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn detects_node_dependencies() {
        let dir = temp_dir();
        write(
            &dir.join("package.json"),
            r#"{
              "dependencies": { "vue": "^3.5.0" },
              "devDependencies": { "vite": "^8.0.0" }
            }"#,
        );

        let report = detect(dir.to_str().unwrap()).unwrap();
        assert_eq!(report.total, 2);
        assert!(
            report
                .dependencies
                .iter()
                .any(|dependency| { dependency.name == "vue" && dependency.kind == "runtime" })
        );
        assert!(
            report
                .dependencies
                .iter()
                .any(|dependency| { dependency.name == "vite" && dependency.kind == "dev" })
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_django_requirements_includes() {
        let dir = temp_dir();
        write(&dir.join("manage.py"), "");
        write(&dir.join("requirements.txt"), "-r requirements/base.txt\n");
        write(
            &dir.join("requirements/base.txt"),
            "Django==5.1.2\ndjangorestframework>=3.15\n",
        );
        write(&dir.join("requirements/dev.txt"), "pytest-django==4.9\n");

        let report = detect(dir.to_str().unwrap()).unwrap();
        assert_eq!(report.framework.as_deref(), Some("Django"));
        assert!(
            report
                .manifests
                .contains(&"requirements/base.txt".to_string())
        );
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "Django" && dependency.version.as_deref() == Some("5.1.2")
        }));
        assert!(
            report.dependencies.iter().any(|dependency| {
                dependency.name == "pytest-django" && dependency.kind == "dev"
            })
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_utf16_requirements() {
        let dir = temp_dir();
        let content = "Django==5.2.6\ndjangorestframework==3.16.1\n";
        let mut bytes = vec![0xFF, 0xFE];
        for unit in content.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        fs::write(dir.join("requirements.txt"), bytes).unwrap();

        let report = detect(dir.to_str().unwrap()).unwrap();
        assert_eq!(report.framework.as_deref(), Some("Django"));
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "Django" && dependency.version.as_deref() == Some("5.2.6")
        }));
        assert!(
            report
                .dependencies
                .iter()
                .any(|dependency| { dependency.name == "djangorestframework" })
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn decodes_utf16_big_endian_and_utf8_bom() {
        let mut utf16 = vec![0xFE, 0xFF];
        for unit in "pandas==2.3.2".encode_utf16() {
            utf16.extend_from_slice(&unit.to_be_bytes());
        }
        let (text, warning) = decode_text(&utf16).unwrap();
        assert_eq!(text, "pandas==2.3.2");
        assert!(warning.is_none());

        let (text, warning) =
            decode_text(&[0xEF, 0xBB, 0xBF, b'D', b'j', b'a', b'n', b'g', b'o']).unwrap();
        assert_eq!(text, "Django");
        assert!(warning.is_none());
    }

    #[test]
    fn detects_cargo_maven_go_and_dotnet() {
        let dir = temp_dir();
        write(
            &dir.join("Cargo.toml"),
            "[dependencies]\nserde = \"1\"\n[dev-dependencies]\ntempfile = \"3\"\n",
        );
        write(
            &dir.join("pom.xml"),
            r#"<project>
              <dependencies>
                <dependency>
                  <groupId>org.springframework.boot</groupId>
                  <artifactId>spring-boot-starter-web</artifactId>
                  <version>4.0.0</version>
                </dependency>
                <dependency>
                  <groupId>org.junit.jupiter</groupId>
                  <artifactId>junit-jupiter</artifactId>
                  <scope>test</scope>
                </dependency>
              </dependencies>
            </project>"#,
        );
        write(
            &dir.join("go.mod"),
            "module example\nrequire (\n  github.com/gin-gonic/gin v1.10.0\n  golang.org/x/tools v0.30.0 // indirect\n)\n",
        );
        write(
            &dir.join("Example.csproj"),
            r#"<Project><ItemGroup>
              <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
            </ItemGroup></Project>"#,
        );

        let report = detect(dir.to_str().unwrap()).unwrap();
        assert!(
            report.dependencies.iter().any(|dependency| {
                dependency.name == "serde" && dependency.ecosystem == "cargo"
            })
        );
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "spring-boot-starter-web" && dependency.ecosystem == "maven"
        }));
        assert!(
            report.dependencies.iter().any(|dependency| {
                dependency.name == "junit-jupiter" && dependency.kind == "dev"
            })
        );
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "golang.org/x/tools" && dependency.kind == "indirect"
        }));
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "Newtonsoft.Json" && dependency.ecosystem == "dotnet"
        }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_gradle_pyproject_and_pipfile() {
        let dir = temp_dir();
        write(
            &dir.join("build.gradle.kts"),
            r#"
            dependencies {
              implementation("org.springframework.boot:spring-boot-starter-web:4.0.0")
              testImplementation("org.junit.jupiter:junit-jupiter:5.11.0")
            }
            "#,
        );
        write(
            &dir.join("pyproject.toml"),
            r#"
            [project]
            dependencies = ["Django>=5.0", "requests[socks]==2.32.3"]

            [project.optional-dependencies]
            dev = ["pytest==8.3.0"]
            "#,
        );
        write(
            &dir.join("Pipfile"),
            r#"
            [packages]
            celery = "*"

            [dev-packages]
            black = "*"
            "#,
        );

        let report = detect(dir.to_str().unwrap()).unwrap();
        assert_eq!(report.framework.as_deref(), Some("Django"));
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "spring-boot-starter-web" && dependency.ecosystem == "gradle"
        }));
        assert!(
            report.dependencies.iter().any(|dependency| {
                dependency.name == "junit-jupiter" && dependency.kind == "dev"
            })
        );
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "Django" && dependency.version.as_deref() == Some(">=5.0")
        }));
        assert!(report.dependencies.iter().any(|dependency| {
            dependency.name == "requests" && dependency.version.as_deref() == Some("2.32.3")
        }));
        assert!(
            report
                .dependencies
                .iter()
                .any(|dependency| { dependency.name == "pytest" && dependency.kind == "dev" })
        );
        assert!(
            report.dependencies.iter().any(|dependency| {
                dependency.name == "celery" && dependency.ecosystem == "python"
            })
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn reports_missing_directory() {
        let report = detect("C:\\path-that-does-not-exist-cardumen-deck").unwrap();
        assert_eq!(report.total, 0);
        assert!(!report.warnings.is_empty());
    }
}
