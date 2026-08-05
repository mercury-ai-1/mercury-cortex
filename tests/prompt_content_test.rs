//! Tests that workflow prompts never instruct the AI to delete temporary
//! metadata files, and that they reflect the engine-owned import model.

use mercury_cortex::mcp::tools::prompts::{dev, init};

fn assert_no_temp_deletion_instructions(content: &str, label: &str) {
    for phrase in [
        "delete that JSON file",
        "Delete any that fail",
        "temp directory is removed",
        "deletes files on success",
        "removes the temp directory",
        "remain in `.mercury-cortex/temp/`",
        "confirm that temporary metadata",
        "Remaining:",
        "No pending temporary metadata",
    ] {
        assert!(
            !content.contains(phrase),
            "{label} must not instruct temp-file deletion, found {phrase:?}"
        );
    }
}

#[test]
fn init_step4_has_no_temp_deletion_instructions() {
    assert_no_temp_deletion_instructions(
        init::step_content(4).expect("init step 4 exists"),
        "init step 4",
    );
}

#[test]
fn init_step5_has_no_temp_deletion_instructions() {
    assert_no_temp_deletion_instructions(
        init::step_content(5).expect("init step 5 exists"),
        "init step 5",
    );
}

#[test]
fn dev_step5_has_no_temp_deletion_instructions() {
    assert_no_temp_deletion_instructions(
        dev::step_content(5).expect("dev step 5 exists"),
        "dev step 5",
    );
}

#[test]
fn dev_step6_has_no_temp_deletion_instructions() {
    assert_no_temp_deletion_instructions(
        dev::step_content(6).expect("dev step 6 exists"),
        "dev step 6",
    );
}

#[test]
fn init_step4_reflects_engine_owned_import() {
    let content = init::step_content(4).expect("init step 4 exists");
    assert!(
        content.contains("metadata/import"),
        "step 4 must use metadata/import"
    );
    assert!(
        content.contains("indexed_files"),
        "step 4 must explain the indexed_files field"
    );
}

#[test]
fn init_step5_reports_engine_counts() {
    let content = init::step_content(5).expect("init step 5 exists");
    assert!(
        content.contains("indexed_files"),
        "step 5 must report indexed_files"
    );
}
