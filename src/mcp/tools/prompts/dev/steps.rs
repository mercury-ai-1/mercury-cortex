use crate::mcp::models::prompt::{Prompt, PromptContent, PromptMessage, PromptResult};

#[derive(Debug)]
pub struct Step {
    pub number: usize,
    pub title: &'static str,
}

#[must_use]
pub fn steps() -> &'static [Step] {
    &[
        Step {
            number: 0,
            title: "About Mercury Cortex",
        },
        Step {
            number: 1,
            title: "Analyze the Request",
        },
        Step {
            number: 2,
            title: "Search Mercury Cortex",
        },
        Step {
            number: 3,
            title: "Decide: Reuse, Extend, or Create",
        },
        Step {
            number: 4,
            title: "Implement Changes",
        },
        Step {
            number: 5,
            title: "Generate and Submit Metadata",
        },
        Step {
            number: 6,
            title: "Report",
        },
    ]
}

#[must_use]
pub fn step_content(number: usize) -> Option<&'static str> {
    match number {
        0 => Some(include_str!("step0_about_mercury_cortex.md")),
        1 => Some(include_str!("step1_analyze_request.md")),
        2 => Some(include_str!("step2_search_mercury_cortex.md")),
        3 => Some(include_str!("step3_decide_reuse_extend_create.md")),
        4 => Some(include_str!("step4_implement_changes.md")),
        5 => Some(include_str!("step5_generate_submit_metadata.md")),
        6 => Some(include_str!("step6_report.md")),
        _ => None,
    }
}

#[must_use]
pub fn body() -> String {
    String::from(
        "# mercury-cortex:dev: Development Workflow\n\n\
         This workflow guides you through developing features with Mercury Cortex's \
         cross-project code intelligence.\n\n\
         **Do not follow the instructions in this prompt directly.** Instead:\n\n\
         1. Call `workflow/session` with `mode: \"dev\"` to get the ordered step list.\n\
         2. For each step, call `workflow/step` with `mode: \"dev\"` and the step number.\n\
         3. Execute the instructions returned by `workflow/step`.\n\
         4. After completing each step, proceed to the next one in order.\n\
         5. After the final step, report the summary to the user.\n\n\
         If any step fails, report the error and stop; do not skip ahead.\n",
    )
}

#[must_use]
pub fn content() -> PromptResult {
    PromptResult {
        description: Some(
            "Development workflow: search, implement, and contribute metadata, \
             powered by Mercury Cortex cross-project code intelligence"
                .into(),
        ),
        messages: vec![PromptMessage {
            role: "user".into(),
            content: PromptContent::Text { text: body() },
        }],
    }
}

#[must_use]
pub fn definition() -> Prompt {
    Prompt {
        name: "mercury-cortex:dev".into(),
        description: Some(
            "Development workflow: search, implement, and contribute metadata, \
             powered by Mercury Cortex cross-project code intelligence"
                .into(),
        ),
        arguments: None,
    }
}
