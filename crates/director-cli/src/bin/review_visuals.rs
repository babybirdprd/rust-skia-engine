use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    println!("Visual Regression Review Tool");
    println!("=============================");

    let root_dir = env::current_dir().expect("Failed to get current dir");
    // Look for failures in crates/director-core/target/visual_regression_failures
    // But since this binary might be run from root, we need to check the path.
    // Assuming standard layout:
    let failure_dir = root_dir.join("crates/director-core/target/visual_regression_failures");
    let snapshot_dir = root_dir.join("crates/director-core/tests/snapshots");

    if !failure_dir.exists() {
        println!("No failure directory found at {:?}.", failure_dir);
        println!("Run tests first: cargo test --test visual_suite");
        return;
    }

    let entries = fs::read_dir(&failure_dir).expect("Failed to read failure dir");
    let mut failures = Vec::new();

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.ends_with("_actual.png") {
                // Determine base name from actual, as diff might not exist (e.g. dimension mismatch)
                let base_name = file_name.trim_end_matches("_actual.png");
                failures.push(base_name.to_string());
            }
        }
    }

    if failures.is_empty() {
        println!("No failures found.");
        return;
    }

    // Sort for consistent order
    failures.sort();

    // Generate HTML Report
    generate_html_report(&failures, &failure_dir, &snapshot_dir);

    // Interactive Mode
    println!("\nFound {} failures.", failures.len());
    print!("Start interactive review? [Y/n]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() == "n" {
        return;
    }

    for fail in failures {
        let diff_path = failure_dir.join(format!("{}_diff.png", fail));
        let actual_path = failure_dir.join(format!("{}_actual.png", fail));
        // The snapshot filename corresponds to the base name (which includes _os suffix already from our test logic)
        let snapshot_path = snapshot_dir.join(format!("{}.png", fail));

        println!("\n------------------------------------------------");
        println!("Failure: {}", fail);
        println!("  Diff:   {:?}", diff_path);
        println!("  Actual: {:?}", actual_path);
        println!("  Ref:    {:?}", snapshot_path);

        // In a real GUI/Advanced CLI, we would show the image.
        // Here we just ask.
        print!("Accept new snapshot? [y/N/q]: ");
        io::stdout().flush().unwrap();

        let mut answer = String::new();
        io::stdin().read_line(&mut answer).unwrap();
        let answer = answer.trim().to_lowercase();

        if answer == "q" {
            break;
        } else if answer == "y" {
            // Move actual to snapshot
            fs::copy(&actual_path, &snapshot_path).expect("Failed to update snapshot");
            println!("Updated: {:?}", snapshot_path);

            // Clean up artifacts for this test
            fs::remove_file(&diff_path).ok();
            fs::remove_file(&actual_path).ok();
        } else {
            println!("Skipped.");
        }
    }

    println!("Review complete.");
}

fn generate_html_report(failures: &[String], failure_dir: &Path, snapshot_dir: &Path) {
    let report_path = failure_dir.parent().unwrap().join("visual_report.html");

    let mut html = String::from(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Visual Regression Report</title>
        <style>
            body { font-family: sans-serif; background: #222; color: #eee; padding: 20px; }
            .test-case { background: #333; margin-bottom: 30px; padding: 20px; border-radius: 8px; }
            h2 { margin-top: 0; }
            .images { display: flex; gap: 20px; flex-wrap: wrap; }
            .image-container { display: flex; flex-direction: column; align-items: center; }
            img { border: 1px solid #555; max-width: 100%; }
            .label { margin-top: 8px; font-weight: bold; color: #ccc; }
        </style>
    </head>
    <body>
        <h1>Visual Regression Failures</h1>
    "#,
    );

    for fail in failures {
        // We need relative paths for HTML if we want it to be portable, but absolute is easier for local dev.
        // Let's use file:// paths or relative if possible.
        // Since report is in target/, relative to target/ is best.
        // failure_dir is target/visual_regression_failures/
        // snapshot_dir is ../../tests/snapshots (relative to target/?) No.

        // Let's just use absolute paths for local viewing simplicity.
        let diff_path = failure_dir.join(format!("{}_diff.png", fail));
        let actual_path = failure_dir.join(format!("{}_actual.png", fail));
        let expected_path = snapshot_dir.join(format!("{}.png", fail));

        let block = format!(
            r#"
        <div class="test-case">
            <h2>{}</h2>
            <div class="images">
                <div class="image-container">
                    <img src="file://{}" width="300" />
                    <span class="label">Expected</span>
                </div>
                <div class="image-container">
                    <img src="file://{}" width="300" />
                    <span class="label">Actual</span>
                </div>
                <div class="image-container">
                    <img src="file://{}" width="300" />
                    <span class="label">Diff</span>
                </div>
            </div>
        </div>
        "#,
            fail,
            expected_path.to_string_lossy(),
            actual_path.to_string_lossy(),
            diff_path.to_string_lossy(),
        );

        html.push_str(&block);
    }

    html.push_str("</body></html>");

    fs::write(&report_path, html).expect("Failed to write HTML report");
    println!("HTML Report generated at: {:?}", report_path);
}
