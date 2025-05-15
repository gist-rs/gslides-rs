use similar::{ChangeTag, TextDiff};
use std::fmt::Write;

/// Holds the results of an SVG comparison, formatted as a Markdown report.
#[derive(Debug)]
pub struct SvgDiffMarkdownReport {
    pub markdown_report: String,
    pub has_differences: bool,
}

/// Compares two SVG content strings and generates a structured Markdown diff report.
///
/// # Arguments
/// * `base_svg_content` - The content of the base SVG file.
/// * `changed_svg_content` - The content of the changed SVG file.
/// * `base_filename` - The name to use for the original file in the diff header and report.
/// * `changed_filename` - The name to use for the new file in the diff header and report.
///
/// # Returns
/// An `SvgDiffMarkdownReport` containing the Markdown report and a flag indicating if differences were found.
pub fn compare_svg_content(
    base_svg_content: &str,
    changed_svg_content: &str,
    base_filename: &str,
    changed_filename: &str,
) -> SvgDiffMarkdownReport {
    let diff = TextDiff::from_lines(base_svg_content, changed_svg_content);
    let mut markdown_report_content = String::new();
    let mut added_lines = 0;
    let mut removed_lines = 0;
    let has_differences = base_svg_content != changed_svg_content;

    // Start Markdown Report
    writeln!(markdown_report_content, "# Summary of SVG Changes\n").expect("Failed to write to string");
    writeln!(markdown_report_content, "---").expect("Failed to write to string");
    writeln!(
        markdown_report_content,
        "## Comparison: `{}` vs `{}`\n",
        base_filename, changed_filename
    )
    .expect("Failed to write to string");

    if has_differences {
        let mut git_diff_hunks = String::new();
        write!(
            git_diff_hunks,
            "{}",
            diff.unified_diff()
                .header(base_filename, changed_filename) // these will be embedded in the ```diff block
                .context_radius(3)
        )
        .expect("Failed to write unified diff to string");

        for change_event in diff.iter_all_changes() {
            match change_event.tag() {
                ChangeTag::Insert => added_lines += 1,
                ChangeTag::Delete => removed_lines += 1,
                ChangeTag::Equal => (),
            }
        }

        writeln!(markdown_report_content, "> SVG files differ.").expect("Failed to write to string");
        writeln!(markdown_report_content, "> - Lines Added: {}", added_lines).expect("Failed to write to string");
        writeln!(markdown_report_content, "> - Lines Removed: {}\n", removed_lines).expect("Failed to write to string");

        writeln!(markdown_report_content, "```diff").expect("Failed to write to string");
        markdown_report_content.push_str(&git_diff_hunks); // Add the actual diff content
        if !git_diff_hunks.ends_with('\n') {
            markdown_report_content.push('\n');
        }
        writeln!(markdown_report_content, "```").expect("Failed to write to string");
    } else {
        writeln!(
            markdown_report_content,
            "> No textual differences found between SVG files."
        )
        .expect("Failed to write to string");
    }

    writeln!(markdown_report_content, "\n---").expect("Failed to write to string");

    SvgDiffMarkdownReport {
        markdown_report: markdown_report_content,
        has_differences,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    // Helper to construct full paths for test files
    fn get_test_file_path(relative_path: &str) -> String {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .join(relative_path)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn test_svg_diff_no_changes() {
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect x="10" y="10" width="80" height="80" fill="blue" />
</svg>"#;
        let result = compare_svg_content(svg_content, svg_content, "a.svg", "b.svg");
        assert!(!result.has_differences);
        assert!(result
            .markdown_report
            .contains("# Summary of SVG Changes"));
        assert!(result
            .markdown_report
            .contains("## Comparison: `a.svg` vs `b.svg`"));
        assert!(result
            .markdown_report
            .contains("> No textual differences found between SVG files."));
        assert!(result.markdown_report.ends_with("---\n"));
    }

    #[test]
    fn test_svg_diff_with_changes() {
        let base_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect x="10" y="10" width="80" height="80" fill="blue" />
</svg>"#;
        let changed_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect x="10" y="10" width="80" height="80" fill="red" />
  <circle cx="50" cy="50" r="10" fill="yellow" />
</svg>"#;
        let result = compare_svg_content(base_svg, changed_svg, "base.svg", "changed.svg");
        assert!(result.has_differences);
        let report = &result.markdown_report;
        assert!(report.contains("# Summary of SVG Changes"));
        assert!(report.contains("## Comparison: `base.svg` vs `changed.svg`"));
        assert!(report.contains("> SVG files differ."));
        assert!(report.contains("> - Lines Added: 2")); // Adjusted for line-based diff
        assert!(report.contains("> - Lines Removed: 1")); // Adjusted for line-based diff
        assert!(report.contains("```diff"));
        assert!(report.contains("--- a/base.svg"));
        assert!(report.contains("+++ b/changed.svg"));
        assert!(report.contains(
            "-  <rect x=\"10\" y=\"10\" width=\"80\" height=\"80\" fill=\"blue\" />"
        ));
        assert!(report.contains(
            "+  <rect x=\"10\" y=\"10\" width=\"80\" height=\"80\" fill=\"red\" />"
        ));
        assert!(report.contains("+  <circle cx=\"50\" cy=\"50\" r=\"10\" fill=\"yellow\" />"));
        assert!(report.contains("```"));
        assert!(report.ends_with("---\n"));
    }

    #[test]
    fn test_with_provided_svg_files() {
        // Paths are relative to the gslides-tools crate root.
        // In a workspace, env!("CARGO_MANIFEST_DIR") points to the crate's root.
        let base_svg_path = get_test_file_path("base_slide_1.svg");
        let diff_svg_path = get_test_file_path("diff_slide_1.svg");

        let base_svg_content = fs::read_to_string(&base_svg_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", base_svg_path, e));
        let diff_svg_content = fs::read_to_string(&diff_svg_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", diff_svg_path, e));

        let result = compare_svg_content(
            &base_svg_content,
            &diff_svg_content,
            "base_slide_1.svg",
            "diff_slide_1.svg",
        );

        assert!(
            result.has_differences,
            "Expected differences between SVG files, but none were found. Report: {}",
            result.markdown_report
        );

        let report = &result.markdown_report;

        assert!(report.contains("# Summary of SVG Changes"));
        assert!(report.contains("## Comparison: `base_slide_1.svg` vs `diff_slide_1.svg`"));
        assert!(report.contains("> SVG files differ."));
        assert!(
            report.contains("> - Lines Added: 2"),
            "Incorrect 'Lines Added' count. Report: {}",
            report
        );
        assert!(
            report.contains("> - Lines Removed: 2"),
            "Incorrect 'Lines Removed' count. Report: {}",
            report
        );

        assert!(report.contains("```diff"));
        assert!(
            report.contains("--- a/base_slide_1.svg"),
            "Git diff header missing for base file. Report: {}",
            report
        );
        assert!(
            report.contains("+++ b/diff_slide_1.svg"),
            "Git diff header missing for changed file. Report: {}",
            report
        );

        // Check for specific content changes known to be in the diff
        // Change 1: color from #000000 to #d2365f
        assert!(report.contains("-<span style=\"font-family:'Noto Sans JP'; font-size:12pt; color:#000000; line-height:12pt;\">あのイーハトーヴォのすきとおった風、夏でも底に冷たさをもつ<br/></span>"), "Missing expected removed line for color change to #000000. Report: {}", report);
        assert!(report.contains("+<span style=\"font-family:'Noto Sans JP'; font-size:12pt; color:#d2365f; line-height:12pt;\">あのイーハトーヴォのすきとおった風、夏でも底に冷たさをもつ<br/></span>"), "Missing expected added line for color change to #d2365f. Report: {}", report);

        // Change 2: text formatting "マスター テキストの書式設定 18pt"
        assert!(report.contains("-<span style=\"font-family:'Noto Sans JP'; font-size:18pt; color:#000000; line-height:18pt;\">マスター テキストの書式設定 18pt<br/></span>"), "Missing expected removed line for text formatting change (base). Report: {}", report);
        assert!(report.contains("+<span style=\"font-family:'Noto Sans JP'; font-size:18pt; color:#000000; line-height:18pt;\">マスター テキストの書式設定 </span><span style=\"font-family:'Noto Sans JP'; font-size:18pt; color:#71cc98; font-weight:bold; line-height:18pt;\">18pt<br/></span>"), "Missing expected added line for text formatting change (diff). Report: {}", report);
        
        assert!(report.contains("```"));
        assert!(report.ends_with("---\n"));
    }
}
