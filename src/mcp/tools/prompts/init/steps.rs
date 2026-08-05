use crate::mcp::models::prompt::{
    Prompt, PromptArgument, PromptContent, PromptMessage, PromptResult,
};

#[derive(Debug)]
pub struct Step {
    pub number: usize,
    pub title: &'static str,
}

#[must_use]
pub fn steps() -> &'static [Step] {
    &[
        Step {
            number: 1,
            title: "Prerequisites and Validation",
        },
        Step {
            number: 2,
            title: "Project Analysis",
        },
        Step {
            number: 3,
            title: ".mcignore Refinement",
        },
        Step {
            number: 4,
            title: "Metadata Generation and Import",
        },
        Step {
            number: 5,
            title: "Verification and Summary",
        },
    ]
}

#[must_use]
pub fn step_content(number: usize) -> Option<&'static str> {
    match number {
        1 => Some(include_str!("step1_validate_project.md")),
        2 => Some(include_str!("step2_analyze_project.md")),
        3 => Some(include_str!("step3_update_mcignore.md")),
        4 => Some(include_str!("step4_generate_metadata.md")),
        5 => Some(include_str!("step5_summary.md")),
        _ => None,
    }
}

#[must_use]
pub fn body() -> String {
    String::from(
        "# mercury-cortex:init — Project Initialization Workflow\n\n\
         This workflow registers the project with Mercury Cortex, analyzes its language and \
         framework, refines `.mcignore`, generates metadata for the project's files, and imports \
         them via `metadata/import`.\n\n\
         **Do not follow the instructions in this prompt directly.** Instead:\n\n\
         1. Call `workflow/session` with `mode: \"init\"` to get the ordered step list.\n\
         2. For each step, call `workflow/step` with `mode: \"init\"` and the step number.\n\
         3. Execute the instructions returned by `workflow/step`.\n\
         4. After completing each step, proceed to the next one in order.\n\
         5. After the final step, report the summary to the user.\n\n\
         If any step fails, report the error and stop — do not skip ahead.\n",
    )
}

#[must_use]
pub fn content() -> PromptResult {
    PromptResult {
        description: Some("Initialize the current project with Mercury Cortex".into()),
        messages: vec![PromptMessage {
            role: "user".into(),
            content: PromptContent::Text { text: body() },
        }],
    }
}

#[must_use]
pub fn definition() -> Prompt {
    Prompt {
        name: "mercury-cortex:init".into(),
        description: Some("Initialize the current project with Mercury Cortex".into()),
        arguments: Some(vec![PromptArgument {
            name: "project_root".into(),
            description: Some("Absolute path to the project root directory".into()),
            required: Some(true),
        }]),
    }
}
