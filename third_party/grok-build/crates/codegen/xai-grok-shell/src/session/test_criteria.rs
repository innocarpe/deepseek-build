//! Host-observed test change evidence from successful structured file writes.
//!
//! This module deliberately stores only relative paths, test names, language tags, and token
//! signatures. It does not retain source text or literal values after parsing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tree_sitter::{Language as TsLanguage, Node, Parser};
use xai_grok_tools::notification::types::FileWritten;

pub(crate) const MAX_SOURCE_BYTES: usize = 1024 * 1024;
const MAX_CRITERIA_PER_FILE: usize = 512;
const MAX_RECEIPT_PATHS_PER_TURN: usize = 64;
const MAX_TEST_NAME_BYTES: usize = 128;
const MAX_RELATIVE_PATH_BYTES: usize = 256;
const MAX_REPORT_ENTRIES: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TestLanguage {
    Rust,
    Go,
    Pytest,
}

impl TestLanguage {
    fn label(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Go => "Go",
            Self::Pytest => "pytest",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TestChangeKind {
    Modified,
    Deleted,
    Added,
    Renamed,
}

impl TestChangeKind {
    fn label(self) -> &'static str {
        match self {
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Added => "added",
            Self::Renamed => "renamed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestCriteriaChange {
    pub path: String,
    pub language: TestLanguage,
    pub change: TestChangeKind,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestCriteriaChangeReport {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<TestCriteriaChange>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub truncated: bool,
}

impl TestCriteriaChangeReport {
    pub(crate) fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Render a host-owned note. Only relative paths, counts, categories, and static scope text
    /// enter this string; test names, source text, literals, commands, and hashes never do.
    pub(crate) fn user_note(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let mut note = String::from(
            "Host-observed test change evidence (successful structured writes on host-backed LocalFs):",
        );
        for change in &self.changes {
            let path = safe_label(&change.path);
            let count = if change.count == 1 {
                format!("{} criterion", change.count)
            } else {
                format!("{} criteria", change.count)
            };
            let previous = match &change.previous_path {
                Some(previous_path) => format!(" from {}", safe_label(previous_path)),
                _ => String::new(),
            };
            note.push_str(&format!(
                "\n- {}: {} ({}, {}){}",
                change.change.label(),
                path,
                change.language.label(),
                count,
                previous
            ));
        }
        if self.truncated {
            note.push_str("\n- Additional evidence omitted by report limits.");
        }
        note.push_str(
            "\nSupported conventions: Rust #[test]/#[tokio::test], Go top-level Test* functions with *testing.T, and pytest module-level test_* functions/Test* class methods in test_*.py or *_test.py files. Formatting and comments are ignored. Shell/MCP/external writes, client-backed ACP writes, non-Git projects, inconsistent receipts, unsupported conventions, parse failures, and oversized files are omitted. Test names and source are not included. This host evidence does not say whether tests ran or passed and does not judge semantic strength.",
        );
        Some(note)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Criterion {
    name: String,
    signature: String,
    language: TestLanguage,
}

#[derive(Debug, Clone)]
pub(crate) struct TestCriteriaWriteReceipt {
    repo_root: PathBuf,
    absolute_path: PathBuf,
    relative_path: String,
    language: TestLanguage,
    before_hash: String,
    after_hash: String,
    before: Vec<Criterion>,
    after: Vec<Criterion>,
}

impl TestCriteriaWriteReceipt {
    /// Convert a `FileWritten` notification into a bounded, source-free receipt. Notifications are
    /// emitted only after the structured writer has written the file successfully.
    pub(crate) fn from_file_written(repo_root: &Path, written: &FileWritten) -> Option<Self> {
        let language = language_for_path(written.absolute_path.as_path())?;
        if written.is_new_file != written.previous_content.is_none() {
            return None;
        }
        let before_text = written.previous_content.as_deref().unwrap_or("");
        if before_text.len() > MAX_SOURCE_BYTES || written.content.len() > MAX_SOURCE_BYTES {
            return None;
        }
        let requested_root = dunce::canonicalize(repo_root).ok()?;
        let repository = git2::Repository::discover(&requested_root).ok()?;
        let workdir = repository.workdir()?;
        let repo_root = dunce::canonicalize(workdir).ok()?;
        if requested_root != repo_root {
            return None;
        }
        let absolute_path = canonical_target_or_parent(&written.absolute_path)?;
        let relative = absolute_path.strip_prefix(&repo_root).ok()?;
        let relative_path = safe_relative_path(relative)?;
        if language_for_path(relative)? != language {
            return None;
        }
        let before = parse_criteria(language, relative, before_text)?;
        let after = parse_criteria(language, relative, &written.content)?;
        Some(Self {
            repo_root,
            absolute_path,
            relative_path,
            language,
            before_hash: content_hash(before_text.as_bytes()),
            after_hash: content_hash(written.content.as_bytes()),
            before,
            after,
        })
    }
}

#[derive(Debug, Clone)]
struct TrackedFile {
    repo_root: PathBuf,
    absolute_path: PathBuf,
    relative_path: String,
    language: TestLanguage,
    initial: Vec<Criterion>,
    current: Vec<Criterion>,
    current_hash: String,
    eligible: bool,
}

#[derive(Debug, Default)]
pub(crate) struct TurnWriteTracker {
    files: HashMap<PathBuf, TrackedFile>,
    truncated: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TurnTestCriteriaState {
    files: Vec<TrackedFile>,
    truncated: bool,
}

impl TurnWriteTracker {
    pub(crate) fn record(&mut self, receipt: TestCriteriaWriteReceipt) {
        if let Some(file) = self.files.get_mut(&receipt.absolute_path) {
            if !file.eligible {
                return;
            }
            if file.current_hash != receipt.before_hash || file.language != receipt.language {
                file.eligible = false;
                return;
            }
            file.current = receipt.after;
            file.current_hash = receipt.after_hash;
            return;
        }
        if self.files.len() >= MAX_RECEIPT_PATHS_PER_TURN {
            self.truncated = true;
            return;
        }
        self.files.insert(
            receipt.absolute_path.clone(),
            TrackedFile {
                repo_root: receipt.repo_root,
                absolute_path: receipt.absolute_path,
                relative_path: receipt.relative_path,
                language: receipt.language,
                initial: receipt.before,
                current: receipt.after,
                current_hash: receipt.after_hash,
                eligible: true,
            },
        );
    }

    pub(crate) fn take(&mut self) -> TurnTestCriteriaState {
        let mut files: Vec<_> = std::mem::take(&mut self.files)
            .into_values()
            .filter(|file| file.eligible)
            .collect();
        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        TurnTestCriteriaState {
            files,
            truncated: std::mem::take(&mut self.truncated),
        }
    }
}

/// Recheck the last successful receipt against the actual path at turn completion before
/// attributing it. This excludes an external edit or unobserved revert after the receipt.
pub(crate) async fn finalize(state: TurnTestCriteriaState) -> TestCriteriaChangeReport {
    let mut valid_files = Vec::new();
    for file in state.files {
        if file_matches_final_content(&file).await {
            valid_files.push(file);
        }
    }
    let mut changes = compare_final_criteria(&valid_files);
    changes.sort_by(|a, b| {
        (a.path.as_str(), a.name.as_str(), a.change as u8).cmp(&(
            b.path.as_str(),
            b.name.as_str(),
            b.change as u8,
        ))
    });
    aggregate_report(changes, state.truncated)
}

async fn file_matches_final_content(file: &TrackedFile) -> bool {
    let absolute_path = file.absolute_path.clone();
    let canonical =
        match tokio::task::spawn_blocking(move || dunce::canonicalize(absolute_path)).await {
            Ok(Ok(path)) => path,
            Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return file.current_hash == content_hash(b"");
            }
            Err(_) => return false,
            Ok(Err(_)) => return false,
        };
    if canonical.strip_prefix(&file.repo_root).is_err() {
        return false;
    }
    let Ok(metadata) = tokio::fs::metadata(&canonical).await else {
        return false;
    };
    if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES as u64 {
        return false;
    }
    let Ok(bytes) = tokio::fs::read(&canonical).await else {
        return false;
    };
    bytes.len() <= MAX_SOURCE_BYTES && content_hash(&bytes) == file.current_hash
}

#[derive(Debug)]
struct DetectedTestChange {
    path: String,
    name: String,
    language: TestLanguage,
    change: TestChangeKind,
    previous_path: Option<String>,
}

fn aggregate_report(
    changes: Vec<DetectedTestChange>,
    mut truncated: bool,
) -> TestCriteriaChangeReport {
    let mut report = TestCriteriaChangeReport::default();
    for change in changes {
        if let Some(existing) = report.changes.iter_mut().find(|existing| {
            existing.path == change.path
                && existing.language == change.language
                && existing.change == change.change
                && existing.previous_path == change.previous_path
        }) {
            existing.count += 1;
            continue;
        }
        if report.changes.len() >= MAX_REPORT_ENTRIES {
            truncated = true;
            continue;
        }
        report.changes.push(TestCriteriaChange {
            path: change.path,
            language: change.language,
            change: change.change,
            count: 1,
            previous_path: change.previous_path,
        });
    }
    report.changes.sort_by(|a, b| {
        (
            a.path.as_str(),
            a.language,
            a.change as u8,
            a.previous_path.as_deref(),
        )
            .cmp(&(
                b.path.as_str(),
                b.language,
                b.change as u8,
                b.previous_path.as_deref(),
            ))
    });
    report.truncated = truncated;
    report
}

fn compare_final_criteria(files: &[TrackedFile]) -> Vec<DetectedTestChange> {
    #[derive(Clone)]
    struct Located {
        path: String,
        criterion: Criterion,
    }

    let mut initial = Vec::new();
    let mut final_state = Vec::new();
    for file in files {
        initial.extend(file.initial.iter().cloned().map(|criterion| Located {
            path: file.relative_path.clone(),
            criterion,
        }));
        final_state.extend(file.current.iter().cloned().map(|criterion| Located {
            path: file.relative_path.clone(),
            criterion,
        }));
    }
    initial.sort_by(|a, b| {
        (&a.path, &a.criterion.name, &a.criterion.signature).cmp(&(
            &b.path,
            &b.criterion.name,
            &b.criterion.signature,
        ))
    });
    final_state.sort_by(|a, b| {
        (&a.path, &a.criterion.name, &a.criterion.signature).cmp(&(
            &b.path,
            &b.criterion.name,
            &b.criterion.signature,
        ))
    });

    let mut remaining_old = Vec::new();
    // An identical test name and signature is the same criterion even if its file moved.
    for old in initial {
        let match_index = final_state.iter().position(|new| {
            old.criterion.language == new.criterion.language
                && old.criterion.name == new.criterion.name
                && old.criterion.signature == new.criterion.signature
        });
        if let Some(index) = match_index {
            final_state.swap_remove(index);
        } else {
            remaining_old.push(old);
        }
    }
    let mut changes = Vec::new();
    // A remaining test with the same name and language is an edit, including a move plus edit.
    let mut old_without_same_name = Vec::new();
    while let Some(old) = remaining_old.pop() {
        let match_index = final_state.iter().position(|new| {
            old.criterion.language == new.criterion.language
                && old.criterion.name == new.criterion.name
        });
        let Some(index) = match_index else {
            old_without_same_name.push(old);
            continue;
        };
        let new = final_state.swap_remove(index);
        changes.push(DetectedTestChange {
            path: new.path.clone(),
            name: new.criterion.name.clone(),
            language: new.criterion.language,
            change: TestChangeKind::Modified,
            previous_path: (old.path != new.path).then(|| old.path.clone()),
        });
    }

    // A matching signature with a different test name is reported as a rename, not a delete/add.
    let mut old_signature_counts = HashMap::new();
    for old in &old_without_same_name {
        *old_signature_counts
            .entry((old.criterion.language, old.criterion.signature.clone()))
            .or_insert(0_usize) += 1;
    }
    let mut old_without_same_signature = Vec::new();
    while let Some(old) = old_without_same_name.pop() {
        let matching_indices: Vec<_> = final_state
            .iter()
            .enumerate()
            .filter_map(|(index, new)| {
                (old.criterion.language == new.criterion.language
                    && old.criterion.signature == new.criterion.signature)
                    .then_some(index)
            })
            .collect();
        let signature_key = (old.criterion.language, old.criterion.signature.clone());
        let old_is_unique = old_signature_counts.get(&signature_key) == Some(&1);
        let [index] = matching_indices.as_slice() else {
            old_without_same_signature.push(old);
            continue;
        };
        if !old_is_unique {
            old_without_same_signature.push(old);
            continue;
        }
        let new = final_state.swap_remove(*index);
        changes.push(DetectedTestChange {
            path: new.path.clone(),
            name: new.criterion.name.clone(),
            language: new.criterion.language,
            change: TestChangeKind::Renamed,
            previous_path: (old.path != new.path).then(|| old.path.clone()),
        });
    }

    for old in old_without_same_signature {
        changes.push(DetectedTestChange {
            path: old.path,
            name: old.criterion.name,
            language: old.criterion.language,
            change: TestChangeKind::Deleted,
            previous_path: None,
        });
    }
    for new in final_state {
        changes.push(DetectedTestChange {
            path: new.path,
            name: new.criterion.name,
            language: new.criterion.language,
            change: TestChangeKind::Added,
            previous_path: None,
        });
    }
    changes
}

fn language_for_path(path: &Path) -> Option<TestLanguage> {
    let name = path.file_name()?.to_str()?;
    if name.ends_with(".rs") {
        Some(TestLanguage::Rust)
    } else if name.ends_with("_test.go") {
        Some(TestLanguage::Go)
    } else if name.ends_with(".py") && (name.starts_with("test_") || name.ends_with("_test.py")) {
        Some(TestLanguage::Pytest)
    } else {
        None
    }
}

fn parse_criteria(language: TestLanguage, path: &Path, source: &str) -> Option<Vec<Criterion>> {
    if source.len() > MAX_SOURCE_BYTES {
        return None;
    }
    let grammar: TsLanguage = match language {
        TestLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
        TestLanguage::Go => tree_sitter_go::LANGUAGE.into(),
        TestLanguage::Pytest => tree_sitter_python::LANGUAGE.into(),
    };
    let mut parser = Parser::new();
    parser.set_language(&grammar).ok()?;
    let tree = parser.parse(source, None)?;
    let root = tree.root_node();
    if root.has_error() {
        return None;
    }
    let bytes = source.as_bytes();
    let mut criteria = Vec::new();
    match language {
        TestLanguage::Rust => collect_rust_criteria(root, &[], bytes, &mut criteria)?,
        TestLanguage::Go => collect_go_criteria(root, bytes, &mut criteria)?,
        TestLanguage::Pytest => collect_pytest_criteria(root, bytes, &mut criteria)?,
    }
    if criteria.len() > MAX_CRITERIA_PER_FILE {
        return None;
    }
    // The extension is intentionally part of the supported convention; keep the parameter
    // visible here so callers cannot accidentally parse a non-test path through this function.
    if language_for_path(path)? != language {
        return None;
    }
    criteria.sort_by(|a, b| (&a.name, &a.signature).cmp(&(&b.name, &b.signature)));
    Some(criteria)
}

fn collect_rust_criteria(
    container: Node<'_>,
    modules: &[String],
    source: &[u8],
    output: &mut Vec<Criterion>,
) -> Option<()> {
    let mut cursor = container.walk();
    for child in container.named_children(&mut cursor) {
        match child.kind() {
            "function_item" if rust_test_attribute(child, source) => {
                let name_node = child.child_by_field_name("name")?;
                let name = node_text(name_node, source)?;
                let attributes = rust_attributes(child);
                let qualified = if modules.is_empty() {
                    name
                } else {
                    format!("{}::{name}", modules.join("::"))
                };
                output.push(Criterion {
                    name: bounded_name(&qualified)?,
                    signature: token_signature(child, name_node, &attributes, source)?,
                    language: TestLanguage::Rust,
                });
            }
            "mod_item" => {
                if let Some(body) = child.child_by_field_name("body") {
                    let mut nested = modules.to_vec();
                    nested.push(node_text(child.child_by_field_name("name")?, source)?);
                    collect_rust_criteria(body, &nested, source, output)?;
                }
            }
            _ => {}
        }
        if output.len() > MAX_CRITERIA_PER_FILE {
            return None;
        }
    }
    Some(())
}

fn rust_test_attribute(item: Node<'_>, source: &[u8]) -> bool {
    rust_attributes(item).into_iter().any(|node| {
        if let Some(attribute) = node_text(node, source) {
            let compact: String = attribute.chars().filter(|c| !c.is_whitespace()).collect();
            if let Some(inner) = compact.strip_prefix("#[").and_then(|s| s.strip_suffix(']')) {
                let path = inner.split(['(', '=']).next().unwrap_or_default();
                return path == "test" || path == "tokio::test";
            }
        }
        false
    })
}

/// Rust attributes are sibling nodes immediately preceding a function item. Keep their source
/// order so attributes that affect test discovery or execution are part of the declaration
/// fingerprint, alongside the function's own syntax tokens.
fn rust_attributes(item: Node<'_>) -> Vec<Node<'_>> {
    let mut attributes = Vec::new();
    let mut previous = item.prev_named_sibling();
    while let Some(node) = previous {
        if node.kind().contains("comment") {
            previous = node.prev_named_sibling();
            continue;
        }
        if node.kind() != "attribute_item" {
            break;
        }
        attributes.push(node);
        previous = node.prev_named_sibling();
    }
    attributes.reverse();
    attributes
}

fn collect_go_criteria(root: Node<'_>, source: &[u8], output: &mut Vec<Criterion>) -> Option<()> {
    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor) {
        if node.kind() != "function_declaration" {
            continue;
        }
        let name_node = node.child_by_field_name("name")?;
        let name = node_text(name_node, source)?;
        if !is_go_test_name(&name) || node.child_by_field_name("body").is_none() {
            continue;
        }
        let Some(parameters) = node.child_by_field_name("parameters") else {
            continue;
        };
        let mut param_cursor = parameters.walk();
        let has_testing_t = parameters
            .named_children(&mut param_cursor)
            .filter(|parameter| parameter.kind() == "parameter_declaration")
            .filter_map(|parameter| parameter.child_by_field_name("type"))
            .filter_map(|ty| node_text(ty, source))
            .any(|ty| compact_whitespace(&ty) == "*testing.T");
        if has_testing_t {
            output.push(Criterion {
                name: bounded_name(&name)?,
                signature: token_signature(node, name_node, &[], source)?,
                language: TestLanguage::Go,
            });
        }
        if output.len() > MAX_CRITERIA_PER_FILE {
            return None;
        }
    }
    Some(())
}

fn is_go_test_name(name: &str) -> bool {
    name.strip_prefix("Test")
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|first| first.is_ascii_uppercase())
}

fn collect_pytest_criteria(
    root: Node<'_>,
    source: &[u8],
    output: &mut Vec<Criterion>,
) -> Option<()> {
    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor) {
        match node.kind() {
            "function_definition" => collect_pytest_function(node, &[], source, output)?,
            "class_definition" => collect_pytest_class(node, &[], source, output)?,
            "decorated_definition" => {
                let (definition, decorators) = unwrap_pytest_definition(node)?;
                match definition.kind() {
                    "function_definition" => {
                        collect_pytest_function(definition, &decorators, source, output)?
                    }
                    "class_definition" => {
                        collect_pytest_class(definition, &decorators, source, output)?
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        if output.len() > MAX_CRITERIA_PER_FILE {
            return None;
        }
    }
    Some(())
}

fn collect_pytest_function(
    node: Node<'_>,
    decorators: &[Node<'_>],
    source: &[u8],
    output: &mut Vec<Criterion>,
) -> Option<()> {
    let name_node = node.child_by_field_name("name")?;
    let name = node_text(name_node, source)?;
    if name.starts_with("test_") {
        output.push(Criterion {
            name: bounded_name(&name)?,
            signature: token_signature(node, name_node, decorators, source)?,
            language: TestLanguage::Pytest,
        });
    }
    Some(())
}

fn collect_pytest_class(
    node: Node<'_>,
    class_decorators: &[Node<'_>],
    source: &[u8],
    output: &mut Vec<Criterion>,
) -> Option<()> {
    let class_name = node_text(node.child_by_field_name("name")?, source)?;
    if !class_name.starts_with("Test") {
        return Some(());
    }
    let Some(body) = node.child_by_field_name("body") else {
        return Some(());
    };
    let mut body_cursor = body.walk();
    let methods: Vec<_> = body.named_children(&mut body_cursor).collect();
    if methods.iter().any(|method| {
        let definition = unwrap_pytest_definition(*method)
            .map(|(definition, _)| definition)
            .unwrap_or(*method);
        definition.kind() == "function_definition"
            && definition
                .child_by_field_name("name")
                .and_then(|name| node_text(name, source))
                .is_some_and(|name| name == "__init__")
    }) {
        return Some(());
    }
    for method in methods {
        let (definition, decorators) = match method.kind() {
            "function_definition" => (method, Vec::new()),
            "decorated_definition" => unwrap_pytest_definition(method)?,
            _ => continue,
        };
        if definition.kind() != "function_definition" {
            continue;
        }
        let method_name_node = definition.child_by_field_name("name")?;
        let method_name = node_text(method_name_node, source)?;
        if !method_name.starts_with("test_") {
            continue;
        }
        let name = format!("{class_name}::{method_name}");
        let mut prefixes = class_decorators.to_vec();
        prefixes.extend(decorators);
        output.push(Criterion {
            name: bounded_name(&name)?,
            signature: token_signature(definition, method_name_node, &prefixes, source)?,
            language: TestLanguage::Pytest,
        });
    }
    Some(())
}

fn unwrap_pytest_definition(node: Node<'_>) -> Option<(Node<'_>, Vec<Node<'_>>)> {
    if node.kind() != "decorated_definition" {
        return Some((node, Vec::new()));
    }
    let mut cursor = node.walk();
    let children: Vec<_> = node.named_children(&mut cursor).collect();
    let definition = children
        .iter()
        .copied()
        .find(|child| matches!(child.kind(), "function_definition" | "class_definition"))?;
    let decorators = children
        .into_iter()
        .take_while(|child| child.id() != definition.id())
        .filter(|child| child.kind() == "decorator")
        .collect();
    Some((definition, decorators))
}

fn token_signature(
    node: Node<'_>,
    name_node: Node<'_>,
    prefixes: &[Node<'_>],
    source: &[u8],
) -> Option<String> {
    fn update(
        node: Node<'_>,
        excluded_range: (usize, usize),
        source: &[u8],
        hasher: &mut blake3::Hasher,
    ) -> Option<()> {
        if (node.start_byte(), node.end_byte()) == excluded_range {
            return Some(());
        }
        if node.kind().contains("comment") {
            return Some(());
        }
        if node.child_count() == 0 {
            let token = source.get(node.byte_range())?;
            hasher.update(&(token.len() as u64).to_le_bytes());
            hasher.update(token);
            return Some(());
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            update(child, excluded_range, source, hasher)?;
        }
        Some(())
    }
    let mut hasher = blake3::Hasher::new();
    for prefix in prefixes {
        update(
            *prefix,
            (name_node.start_byte(), name_node.end_byte()),
            source,
            &mut hasher,
        )?;
    }
    update(
        node,
        (name_node.start_byte(), name_node.end_byte()),
        source,
        &mut hasher,
    )?;
    Some(hasher.finalize().to_hex().to_string())
}

fn node_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    Some(
        std::str::from_utf8(source.get(node.byte_range())?)
            .ok()?
            .to_owned(),
    )
}

fn bounded_name(name: &str) -> Option<String> {
    (name.len() <= MAX_TEST_NAME_BYTES).then(|| safe_label(name))
}

fn compact_whitespace(value: &str) -> String {
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

fn safe_relative_path(path: &Path) -> Option<String> {
    let rendered = path
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    if rendered.is_empty() || rendered.len() > MAX_RELATIVE_PATH_BYTES {
        return None;
    }
    Some(safe_label(&rendered))
}

fn safe_label(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() || matches!(character, '`' | '\n' | '\r') {
                '?'
            } else {
                character
            }
        })
        .collect()
}

fn canonical_target_or_parent(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }
    if let Ok(canonical) = dunce::canonicalize(path) {
        return Some(canonical);
    }
    let parent = path.parent()?;
    let filename = path.file_name()?;
    Some(dunce::canonicalize(parent).ok()?.join(filename))
}

fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn written(root: &Path, name: &str, before: Option<&str>, after: &str) -> FileWritten {
        FileWritten {
            tool_call_id: "test-call".to_owned(),
            absolute_path: root.join(name),
            content: after.to_owned(),
            previous_content: before.map(str::to_owned),
            is_new_file: before.is_none(),
        }
    }

    fn git_tempdir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        dir
    }

    fn record(
        tracker: &mut TurnWriteTracker,
        root: &Path,
        name: &str,
        before: Option<&str>,
        after: &str,
    ) {
        let write = written(root, name, before, after);
        if let Some(receipt) = TestCriteriaWriteReceipt::from_file_written(root, &write) {
            tracker.record(receipt);
        }
    }

    async fn report_for(tracker: &mut TurnWriteTracker) -> TestCriteriaChangeReport {
        finalize(tracker.take()).await
    }

    const RUST_BEFORE: &str = "#[test]\nfn keeps_contract() { assert_eq!(1, 2); }\n";
    const RUST_AFTER: &str = "#[test]\nfn keeps_contract() { assert_eq!(1, 3); }\n";

    #[test]
    fn rust_recognizes_test_and_tokio_test_and_ignores_comments_and_spacing() {
        let path = Path::new("src/change_test.rs");
        let before = "#[test]\nfn sync_case() { assert_eq!(1, 2); }\n#[tokio::test]\nasync fn async_case() { assert!(true); }\n";
        let after = "// comment\n#[test]\nfn sync_case() {\n  assert_eq!(1, 2); // still the same tokens\n}\n#[tokio::test]\nasync fn async_case() { assert!(true); }\n";
        let old = parse_criteria(TestLanguage::Rust, path, before).unwrap();
        let new = parse_criteria(TestLanguage::Rust, path, after).unwrap();
        assert_eq!(old.len(), 2);
        assert_eq!(old, new);
    }

    #[test]
    fn go_and_pytest_follow_the_declared_default_conventions() {
        let go = "package sample\nimport \"testing\"\nfunc TestWorks(t *testing.T) { t.Fatal(\"secret-literal\") }\nfunc TestBad(t int) {}\nfunc BenchmarkNo(b *testing.B) {}\n";
        let go_tests = parse_criteria(TestLanguage::Go, Path::new("sample_test.go"), go).unwrap();
        assert_eq!(
            go_tests
                .iter()
                .map(|test| test.name.as_str())
                .collect::<Vec<_>>(),
            ["TestWorks"]
        );

        let python = "def test_top():\n    assert True\n\nclass TestGroup:\n    def test_method(self):\n        assert True\nclass TestIgnored:\n    def __init__(self): pass\n    def test_method(self): assert True\n";
        let py_tests =
            parse_criteria(TestLanguage::Pytest, Path::new("test_sample.py"), python).unwrap();
        assert_eq!(
            py_tests
                .iter()
                .map(|test| test.name.as_str())
                .collect::<Vec<_>>(),
            ["TestGroup::test_method", "test_top"]
        );
    }

    #[test]
    fn pytest_decorators_and_rust_attributes_are_part_of_declaration_signatures() {
        let python_before = "@pytest.mark.parametrize('value', [1], ids=('before',))\ndef test_parameterized(value):\n    assert value\n\nclass TestGroup:\n    @pytest.mark.skip(reason='before')\n    def test_skipped(self):\n        assert True\n";
        let python_after = "@pytest.mark.parametrize('value', [2], ids=('after',))\ndef test_parameterized(value):\n    assert value\n\nclass TestGroup:\n    @pytest.mark.skip(reason='after')\n    def test_skipped(self):\n        assert True\n";
        let before_py = parse_criteria(
            TestLanguage::Pytest,
            Path::new("test_decorated.py"),
            python_before,
        )
        .unwrap();
        let after_py = parse_criteria(
            TestLanguage::Pytest,
            Path::new("test_decorated.py"),
            python_after,
        )
        .unwrap();
        assert_eq!(before_py.len(), 2);
        assert_eq!(after_py.len(), 2);
        for (before, after) in before_py.iter().zip(&after_py) {
            assert_eq!(before.name, after.name);
            assert_ne!(
                before.signature, after.signature,
                "decorator token edits change the declaration fingerprint"
            );
        }

        let rust_before = "#[test]\n#[ignore]\nfn sync_case() {}\n#[tokio::test(flavor = \"current_thread\")]\nasync fn async_case() {}\n";
        let rust_after = "#[test]\n#[should_panic]\nfn sync_case() {}\n#[tokio::test(flavor = \"multi_thread\", worker_threads = 2)]\nasync fn async_case() {}\n";
        let before_rust =
            parse_criteria(TestLanguage::Rust, Path::new("change_test.rs"), rust_before).unwrap();
        let after_rust =
            parse_criteria(TestLanguage::Rust, Path::new("change_test.rs"), rust_after).unwrap();
        assert_eq!(before_rust.len(), 2);
        assert_eq!(after_rust.len(), 2);
        for (before, after) in before_rust.iter().zip(&after_rust) {
            assert_eq!(before.name, after.name);
            assert_ne!(
                before.signature, after.signature,
                "Rust attribute token edits change the declaration fingerprint"
            );
        }
    }

    #[tokio::test]
    async fn only_successful_receipts_are_attributed_and_preexisting_dirty_state_is_baseline() {
        let dir = git_tempdir();
        let repository = git2::Repository::open(dir.path()).unwrap();
        let test_path = dir.path().join("existing_test.rs");
        let head_content = "#[test]\nfn committed_case() { assert_eq!(1, 1); }\n";
        fs::write(&test_path, head_content).unwrap();
        let tree_id = {
            let mut index = repository.index().unwrap();
            index.add_path(Path::new("existing_test.rs")).unwrap();
            index.write().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repository.find_tree(tree_id).unwrap();
        let signature = git2::Signature::now("test", "test@example.invalid").unwrap();
        repository
            .commit(Some("HEAD"), &signature, &signature, "baseline", &tree, &[])
            .unwrap();
        fs::write(&test_path, RUST_BEFORE).unwrap();

        // The file can already differ from HEAD at turn start. With no successful write receipt,
        // there is no attribution.
        let mut no_receipt = TurnWriteTracker::default();
        assert!(report_for(&mut no_receipt).await.is_empty());

        // The dirty content supplied as previous_content is the baseline, not a HEAD snapshot.
        let mut tracker = TurnWriteTracker::default();
        record(
            &mut tracker,
            dir.path(),
            "existing_test.rs",
            Some(RUST_BEFORE),
            RUST_AFTER,
        );
        fs::write(&test_path, RUST_AFTER).unwrap();
        let report = report_for(&mut tracker).await;
        assert_eq!(report.changes.len(), 1);
        assert!(
            report
                .changes
                .first()
                .is_some_and(|change| change.path == "existing_test.rs"
                    && change.change == TestChangeKind::Modified)
        );

        // A failed write with no FileWritten receipt produces no report.
        let mut failed_write = TurnWriteTracker::default();
        assert!(report_for(&mut failed_write).await.is_empty());
    }

    #[tokio::test]
    async fn reports_modified_deleted_and_added_tests_without_source_or_literals() {
        let dir = git_tempdir();
        let before =
            "#[test]\nfn changed_case() { assert_eq!(1, 2); }\n#[test]\nfn removed_case() {}\n";
        let after = "#[test]\nfn changed_case() { assert_eq!(1, 3); }\n#[test]\nfn added_case() { assert_eq!(\"secret-value\", \"x\"); }\n";
        let path = dir.path().join("tests.rs");
        fs::write(&path, after).unwrap();
        let mut tracker = TurnWriteTracker::default();
        record(&mut tracker, dir.path(), "tests.rs", Some(before), after);
        let report = report_for(&mut tracker).await;
        assert!(
            report.changes.iter().any(
                |change| change.path == "tests.rs" && change.change == TestChangeKind::Modified
            )
        );
        assert!(report
            .changes
            .iter()
            .any(|change| change.path == "tests.rs" && change.change == TestChangeKind::Deleted));
        assert!(
            report
                .changes
                .iter()
                .any(|change| change.path == "tests.rs" && change.change == TestChangeKind::Added)
        );
        let serialized = serde_json::to_string(&report).unwrap();
        assert!(!serialized.contains("secret-value"));
        assert!(!serialized.contains("assert_eq"));
        assert!(!serialized.contains("changed_case"));
        assert!(!report.user_note().unwrap().contains("secret-value"));
    }

    #[tokio::test]
    async fn uniquely_renamed_test_is_reported_without_exposing_its_name() {
        let dir = git_tempdir();
        let before = "#[test]\nfn previous_name() { assert!(true); }\n";
        let after = "#[test]\nfn current_name() { assert!(true); }\n";
        fs::write(dir.path().join("rename_test.rs"), after).unwrap();
        let mut tracker = TurnWriteTracker::default();
        record(
            &mut tracker,
            dir.path(),
            "rename_test.rs",
            Some(before),
            after,
        );

        let report = report_for(&mut tracker).await;
        assert!(report.changes.iter().any(|change| {
            change.path == "rename_test.rs" && change.change == TestChangeKind::Renamed
        }));
        let serialized = serde_json::to_string(&report).unwrap();
        assert!(!serialized.contains("previous_name"));
        assert!(!serialized.contains("current_name"));
    }

    #[tokio::test]
    async fn net_zero_revert_and_test_file_rename_do_not_report_delete_add_pairs() {
        let dir = git_tempdir();
        let original = "#[test]\nfn stable_case() { assert!(true); }\n";
        let changed = "#[test]\nfn stable_case() { assert!(false); }\n";
        let old_path = dir.path().join("before_test.rs");
        let new_path = dir.path().join("after_test.rs");

        fs::write(&old_path, original).unwrap();
        let mut reverted = TurnWriteTracker::default();
        record(
            &mut reverted,
            dir.path(),
            "before_test.rs",
            Some(original),
            changed,
        );
        record(
            &mut reverted,
            dir.path(),
            "before_test.rs",
            Some(changed),
            original,
        );
        fs::write(&old_path, original).unwrap();
        assert!(report_for(&mut reverted).await.is_empty());

        let mut moved = TurnWriteTracker::default();
        record(&mut moved, dir.path(), "before_test.rs", Some(original), "");
        record(&mut moved, dir.path(), "after_test.rs", None, original);
        fs::remove_file(&old_path).unwrap();
        fs::write(&new_path, original).unwrap();
        assert!(report_for(&mut moved).await.is_empty());
    }

    #[tokio::test]
    async fn mismatched_receipt_chain_and_final_external_change_are_omitted() {
        let dir = git_tempdir();
        let changed = "#[test]\nfn case_one() { assert!(false); }\n";
        let external = "#[test]\nfn external_case() { assert!(true); }\n";
        let path = dir.path().join("tests.rs");
        fs::write(&path, external).unwrap();
        let mut tracker = TurnWriteTracker::default();
        record(
            &mut tracker,
            dir.path(),
            "tests.rs",
            Some(RUST_BEFORE),
            changed,
        );
        // The final file no longer matches the successful receipt, so the write cannot be
        // attributed as the final turn state.
        assert!(report_for(&mut tracker).await.is_empty());

        let mut chain = TurnWriteTracker::default();
        record(
            &mut chain,
            dir.path(),
            "tests.rs",
            Some(RUST_BEFORE),
            changed,
        );
        record(
            &mut chain,
            dir.path(),
            "tests.rs",
            Some(external),
            RUST_AFTER,
        );
        fs::write(&path, RUST_AFTER).unwrap();
        assert!(report_for(&mut chain).await.is_empty());
    }

    #[test]
    fn unsupported_paths_parse_errors_and_oversized_files_are_unavailable() {
        assert!(parse_criteria(TestLanguage::Go, Path::new("not.go"), "").is_none());
        assert!(
            parse_criteria(TestLanguage::Go, Path::new("bad_test.go"), "func TestX(").is_none()
        );
        assert!(
            parse_criteria(
                TestLanguage::Rust,
                Path::new("large.rs"),
                &" ".repeat(MAX_SOURCE_BYTES + 1)
            )
            .is_none()
        );
    }

    #[tokio::test]
    async fn oversized_report_is_bounded_and_marks_truncation() {
        let dir = git_tempdir();
        let mut tracker = TurnWriteTracker::default();
        let before = "package sample\nimport \"testing\"\n";
        for index in 0..(MAX_REPORT_ENTRIES + 1) {
            let name = format!("many_{index}_test.go");
            let after =
                format!("{before}func TestCase{index}(t *testing.T) {{ t.Fatal({index}) }}\n");
            fs::write(dir.path().join(&name), &after).unwrap();
            record(&mut tracker, dir.path(), &name, Some(before), &after);
        }
        let report = report_for(&mut tracker).await;
        assert_eq!(report.changes.len(), MAX_REPORT_ENTRIES);
        assert!(report.truncated);
    }

    #[test]
    fn non_git_projects_do_not_produce_write_receipts() {
        let dir = tempfile::tempdir().unwrap();
        let write = written(dir.path(), "tests.rs", Some(RUST_BEFORE), RUST_AFTER);
        assert!(TestCriteriaWriteReceipt::from_file_written(dir.path(), &write).is_none());
    }

    #[test]
    fn inconsistent_new_file_flag_and_missing_previous_content_are_unavailable() {
        let dir = git_tempdir();
        let mut write = written(
            dir.path(),
            "test_new.py",
            None,
            "def test_added():\n    assert True\n",
        );
        write.is_new_file = false;

        assert!(TestCriteriaWriteReceipt::from_file_written(dir.path(), &write).is_none());
    }
}
