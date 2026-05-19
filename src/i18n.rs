// Kalk — your academic dashboard in the terminal.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

//! Internationalization (i18n) support for Kalk.
//!
//! This module provides translations for all UI text in the application.
//! Currently supports English (default) and Spanish.

use serde::{Deserialize, Serialize};

/// Available languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    /// Get the display name of the language.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
        }
    }

    /// Get all available languages.
    pub fn all() -> &'static [Language] {
        &[Language::English, Language::Spanish]
    }
}

/// All translatable messages in the application.
#[derive(Debug, Clone)]
pub struct Messages {
    // Panel titles
    pub courses: &'static str,
    pub categories: &'static str,
    pub evaluations: &'static str,

    // Course panel
    pub select_course_to_view: &'static str,
    pub select_category_to_view: &'static str,
    pub select_course_first: &'static str,
    pub course_average: &'static str,
    pub no_grades_yet: &'static str,
    pub passed: &'static str,
    pub failed: &'static str,
    pub current: &'static str,
    pub need: &'static str,
    pub cannot_pass: &'static str,
    pub no_evaluations: &'static str,

    // Weight validation
    pub weights_ok: &'static str,
    pub weights_warning: &'static str,
    pub weights_error: &'static str,
    pub no_categories: &'static str,

    // Footer / Keybindings
    pub quit: &'static str,
    pub new: &'static str,
    pub new_category: &'static str,
    pub new_eval: &'static str,
    pub edit: &'static str,
    pub delete: &'static str,
    pub save_as_template: &'static str,
    pub balance: &'static str,
    pub yank: &'static str,
    pub paste: &'static str,
    pub bulk_add: &'static str,
    pub select: &'static str,
    pub confirm: &'static str,
    pub cancel: &'static str,
    pub next_field: &'static str,
    pub change_language: &'static str,
    /// Footer label for the `g` shortcut that opens the global-grade input.
    pub enter_global: &'static str,
    /// Footer label for the Home/End jump shortcuts (combined "First / Last").
    pub jump_first_last: &'static str,

    // Popups
    pub select_course_template: &'static str,
    pub new_course: &'static str,
    pub edit_course: &'static str,
    pub new_category_title: &'static str,
    pub edit_category: &'static str,
    pub new_evaluation: &'static str,
    pub edit_evaluation: &'static str,
    pub confirm_delete: &'static str,
    pub delete_course_question: &'static str,
    pub delete_category_question: &'static str,
    pub delete_evaluation_question: &'static str,
    pub delete_course_warning: &'static str,
    pub delete_category_warning: &'static str,
    pub delete_evaluation_warning: &'static str,
    pub save_as_template_title: &'static str,
    pub delete_template: &'static str,
    pub delete_template_question: &'static str,
    pub action_cannot_be_undone: &'static str,
    pub select_language: &'static str,
    pub bulk_add_title: &'static str,
    pub bulk_add_hint: &'static str,
    pub bulk_add_count: &'static str,

    // Form labels
    pub name: &'static str,
    pub passing_grade: &'static str,
    pub weight: &'static str,
    pub grade: &'static str,
    pub template_name: &'static str,
    pub description: &'static str,
    pub current_total: &'static str,
    pub will_save_categories: &'static str,
    pub need_to_pass: &'static str,
    pub graded: &'static str,
    pub avg: &'static str,

    // Template category names
    pub tpl_exam: &'static str,
    pub tpl_quiz: &'static str,
    pub tpl_homework: &'static str,
    pub tpl_lab: &'static str,
    pub tpl_project: &'static str,
    pub tpl_assessment: &'static str,

    // Template names and descriptions
    pub tpl_exam_quiz_80_20: &'static str,
    pub tpl_exam_quiz_80_20_desc: &'static str,
    pub tpl_exam_quiz_70_30: &'static str,
    pub tpl_exam_quiz_70_30_desc: &'static str,
    pub tpl_exam_homework_quiz: &'static str,
    pub tpl_exam_homework_quiz_desc: &'static str,
    pub tpl_exam_labs: &'static str,
    pub tpl_exam_labs_desc: &'static str,
    pub tpl_project_exams: &'static str,
    pub tpl_project_exams_desc: &'static str,
    pub tpl_exams_only: &'static str,
    pub tpl_exams_only_desc: &'static str,
    pub tpl_continuous: &'static str,
    pub tpl_continuous_desc: &'static str,
    pub tpl_custom: &'static str,
    pub tpl_custom_desc: &'static str,

    // Status messages
    pub save_error: &'static str,
    pub save_template_error: &'static str,
    pub config_save_error: &'static str,
    pub load_error: &'static str,
    pub load_template_error: &'static str,
    pub config_load_warning: &'static str,

    // Misc
    pub unknown: &'static str,
    pub empty_template_desc: &'static str,
    pub template_suffix: &'static str,
    pub passing: &'static str,
    pub below_passing: &'static str,
    pub need_grade_impossible: &'static str,
    pub need_grade_any: &'static str,
    pub need_grade_in_eval: &'static str,

    // Category rules labels
    pub advanced_rules: &'static str,
    pub advanced_rules_show: &'static str,
    pub advanced_rules_hide: &'static str,
    pub drop_lowest: &'static str,
    pub averaging_method: &'static str,
    pub averaging_arithmetic: &'static str,
    pub averaging_geometric: &'static str,
    pub minimum_average: &'static str,
    pub on_minimum_not_met: &'static str,
    pub action_final_equals_avg: &'static str,
    pub action_requires_global: &'static str,
    pub action_fail_course: &'static str,
    pub minimum_per_evaluation: &'static str,
    pub minimum_one_eval: &'static str,
    pub round_before_weighting: &'static str,
    pub weighted_evaluations: &'static str,
    pub eval_weight: &'static str,
    pub yes: &'static str,
    pub no: &'static str,

    // Field help descriptions (shown in help panel)
    pub help_toggle: &'static str,
    pub help_name: &'static str,
    pub help_weight: &'static str,
    pub help_drop_lowest: &'static str,
    pub help_averaging_method: &'static str,
    pub help_minimum_average: &'static str,
    pub help_on_min_not_met: &'static str,
    pub help_min_per_eval: &'static str,
    pub help_min_one_eval: &'static str,
    pub help_round_before_weighting: &'static str,
    pub help_weighted_evaluations: &'static str,
    pub help_passing_grade: &'static str,
    pub help_global_policy: &'static str,
    pub help_global_weights: &'static str,
    pub help_global_eligibility: &'static str,

    // Status indicators for rules
    pub rules_active: &'static str,
    pub minimum_not_met: &'static str,
    pub needs_global: &'static str,
    pub grade_capped_by: &'static str,
    pub dropped: &'static str,
    pub eval_below_min: &'static str,
    pub eval_weights_ok: &'static str,
    pub eval_weights_warning: &'static str,
    pub eval_weights_error: &'static str,

    // Global exam
    pub global_exam: &'static str,
    pub global_policy: &'static str,
    pub global_policy_none: &'static str,
    pub global_policy_weighted: &'static str,
    pub global_policy_replaces: &'static str,
    pub global_semester_weight: &'static str,
    pub global_exam_weight: &'static str,
    pub global_min_grade: &'static str,
    pub global_grade: &'static str,
    pub global_needed: &'static str,
    pub global_result: &'static str,
    pub global_no_policy: &'static str,
    pub global_enter_hint: &'static str,

    // Settings
    pub settings: &'static str,
    pub settings_nerd_fonts: &'static str,
    pub settings_nerd_fonts_desc: &'static str,
    pub settings_language: &'static str,
    pub settings_compact_courses: &'static str,
    pub settings_compact_courses_desc: &'static str,
    pub compact: &'static str,
    pub enabled: &'static str,
    pub disabled: &'static str,
    pub toggle: &'static str,
    pub yanked_eval: &'static str,
    pub pasted_eval: &'static str,
    pub no_eval_to_yank: &'static str,
    pub no_eval_in_clipboard: &'static str,
    pub bulk_added_evals: &'static str,

    // Undo / redo
    pub undo: &'static str,
    pub redo: &'static str,
    pub undo_done: &'static str,
    pub redo_done: &'static str,
    pub nothing_to_undo: &'static str,
    pub nothing_to_redo: &'static str,

    // Delete impact (singular / plural forms)
    pub eval_singular: &'static str,
    pub eval_plural: &'static str,
    pub category_singular: &'static str,
    pub category_plural: &'static str,

    // Help overlay
    pub help_title: &'static str,
    pub help_close_hint: &'static str,
    pub help_open: &'static str,
    pub help_group_global: &'static str,
    pub help_group_navigation: &'static str,
    pub help_group_editing: &'static str,
    pub help_group_actions: &'static str,
    pub help_group_view: &'static str,
    pub help_cycle_focus: &'static str,
    pub help_focus_lr: &'static str,
    pub help_move_updown: &'static str,
    pub help_close_popup: &'static str,
    pub help_new_generic: &'static str,
    pub help_edit_selected: &'static str,
    pub help_delete_selected: &'static str,

    // AI import wizard
    pub tpl_ai: &'static str,
    pub tpl_ai_desc: &'static str,
    pub import_step1_title: &'static str,
    pub import_step1_hint: &'static str,
    pub import_step1_copy_key: &'static str,
    pub import_step1_copied: &'static str,
    pub import_step1_next: &'static str,
    pub import_step1_fullscreen: &'static str,
    pub import_step1_fullscreen_exit: &'static str,
    pub import_step2_title: &'static str,
    pub import_step2_hint: &'static str,
    pub import_step2_back: &'static str,
    pub import_step3_title: &'static str,
    pub import_step3_total_weight: &'static str,
    pub import_step3_evaluations: &'static str,
    pub import_step3_save_as_template: &'static str,
    pub import_step3_confirm: &'static str,
    pub import_warning_weight_not_100: &'static str,
    pub import_err_invalid_json: &'static str,
    pub import_err_schema_version: &'static str,
    pub import_err_empty_name: &'static str,
    pub import_err_no_categories: &'static str,
    pub import_imported_ok: &'static str,
    pub import_copy_suffix: &'static str,
    pub import_renamed_to: &'static str,
    /// Full prompt template handed to the AI (raw multi-line string).
    pub import_prompt: &'static str,
}

/// English messages.
pub const EN: Messages = Messages {
    // Panel titles
    courses: "Courses",
    categories: "Categories",
    evaluations: "Evaluations",

    // Course panel
    select_course_to_view: "Select a course to view categories.",
    select_category_to_view: "Select a category to view evaluations.",
    select_course_first: "Select a course first.",
    course_average: "Course Average",
    no_grades_yet: "No grades yet",
    passed: "PASSED",
    failed: "FAILED",
    current: "Current",
    need: "Need",
    cannot_pass: "Cannot pass",
    no_evaluations: "No evaluations",

    // Weight validation
    weights_ok: "OK",
    weights_warning: "Only",
    weights_error: "Err:",
    no_categories: "No categories",

    // Footer / Keybindings
    quit: "Quit",
    new: "New Course",
    new_category: "New Category",
    new_eval: "New Evaluation",
    edit: "Edit",
    delete: "Delete",
    save_as_template: "Save Template",
    balance: "Balance Weights",
    yank: "Copy",
    paste: "Paste",
    bulk_add: "Add Many",
    select: "Select",
    confirm: "Confirm",
    cancel: "Cancel",
    next_field: "Next field",
    change_language: "Language",
    enter_global: "Enter Global Grade",
    jump_first_last: "First / Last",

    // Popups
    select_course_template: "Select Course Template",
    new_course: "New Course",
    edit_course: "Edit Course",
    new_category_title: "New Category",
    edit_category: "Edit Category",
    new_evaluation: "New Evaluation",
    edit_evaluation: "Edit Evaluation",
    confirm_delete: "Confirm Delete",
    delete_course_question: "Delete this course?",
    delete_category_question: "Delete this category?",
    delete_evaluation_question: "Delete this evaluation?",
    delete_course_warning: "All categories and evaluations will be lost.",
    delete_category_warning: "All evaluations in this category will be lost.",
    delete_evaluation_warning: "This action cannot be undone.",
    save_as_template_title: "Save as Template",
    delete_template: "Delete Template",
    delete_template_question: "Delete",
    action_cannot_be_undone: "This action cannot be undone.",
    select_language: "Select Language",
    bulk_add_title: "Bulk Add Evaluations",
    bulk_add_hint: "Creates Name 1, Name 2, ... Name N",
    bulk_add_count: "Count",

    // Form labels
    name: "Name",
    passing_grade: "Passing Grade (0-100)",
    weight: "Weight % (e.g., 80)",
    grade: "Grade (0-100)",
    template_name: "Template Name",
    description: "Description",
    current_total: "Current total",
    will_save_categories: "Will save categories from",
    need_to_pass: "Need to Pass",
    graded: "graded",
    avg: "Avg",

    // Template category names
    tpl_exam: "Exam",
    tpl_quiz: "Quiz",
    tpl_homework: "Homework",
    tpl_lab: "Lab",
    tpl_project: "Project",
    tpl_assessment: "Assessment",

    // Template names and descriptions
    tpl_exam_quiz_80_20: "Exams + Quizzes (80/20)",
    tpl_exam_quiz_80_20_desc: "3 Exams (80%) + 3 Quizzes (20%)",
    tpl_exam_quiz_70_30: "Exams + Quizzes (70/30)",
    tpl_exam_quiz_70_30_desc: "3 Exams (70%) + 3 Quizzes (30%)",
    tpl_exam_homework_quiz: "Exams + Homework + Quizzes",
    tpl_exam_homework_quiz_desc: "3 Exams (60%) + 4 Homework (25%) + 3 Quizzes (15%)",
    tpl_exam_labs: "Exams + Labs",
    tpl_exam_labs_desc: "3 Exams (70%) + 6 Labs (30%)",
    tpl_project_exams: "Project + Exams",
    tpl_project_exams_desc: "Project (40%) + 2 Exams (60%)",
    tpl_exams_only: "Exams Only",
    tpl_exams_only_desc: "3 Exams (100%)",
    tpl_continuous: "Continuous Assessment",
    tpl_continuous_desc: "10 Weekly assessments (100%)",
    tpl_custom: "Custom (Empty)",
    tpl_custom_desc: "Start with no categories - add your own",

    // Status messages
    save_error: "Error: Failed to save data",
    save_template_error: "Error: Failed to save templates",
    config_save_error: "Error: Failed to save config",
    load_error: "Warning: Failed to load data, starting fresh",
    load_template_error: "Warning: Failed to load templates",
    config_load_warning: "Warning: Failed to load config, using defaults",

    // Misc
    unknown: "Unknown",
    empty_template_desc: "Empty template",
    template_suffix: "Template",
    passing: "passing",
    below_passing: "below passing",
    need_grade_impossible: "impossible, max is 100",
    need_grade_any: "already passing with any grade",
    need_grade_in_eval: "in this eval to pass",

    // Category rules labels
    advanced_rules: "Advanced Rules",
    advanced_rules_show: "Show rules",
    advanced_rules_hide: "Hide rules",
    drop_lowest: "Drop Lowest",
    averaging_method: "Average Method",
    averaging_arithmetic: "Arithmetic",
    averaging_geometric: "Geometric",
    minimum_average: "Min. Average",
    on_minimum_not_met: "If Not Met",
    action_final_equals_avg: "Final = Category Avg",
    action_requires_global: "Requires Global",
    action_fail_course: "Fail Course",
    minimum_per_evaluation: "Min. Per Eval",
    minimum_one_eval: "Min. One Eval >=",
    round_before_weighting: "Round Category",
    weighted_evaluations: "Weighted Evals",
    eval_weight: "Eval Weight %",
    yes: "Yes",
    no: "No",

    // Field help descriptions
    help_toggle: "Help",
    help_name: "Category label (e.g. Exams, Quizzes, Labs).",
    help_weight: "How much this category counts toward the final grade (0-100%).",
    help_drop_lowest: "Discard the N worst grades before averaging. 0 = keep all.",
    help_averaging_method: "Arithmetic = normal average. Geometric = root-product average (useful for multiplicative grading).",
    help_on_min_not_met: "What happens when the minimum average is not reached. Final=Avg means the course grade equals this category average. Requires Global means you must take a global exam. Fail Course means the course is automatically failed.",
    help_minimum_average: "Minimum average required in this category (e.g. 55). Leave empty for none.",
    help_min_per_eval: "Minimum grade required on each individual evaluation. Leave empty for none.",
    help_min_one_eval: "At least one graded evaluation must reach this grade (e.g. 50). If none does, the 'If Not Met' action triggers. Leave empty for none.",
    help_round_before_weighting: "Round the category average before applying weight. Useful when the university rounds per category.",
    help_weighted_evaluations: "When enabled, each evaluation has its own weight (must sum to 100%). Useful when a category has sub-components with different weights (e.g. report 40%, presentation 60%).",
    help_passing_grade: "Minimum grade to pass the course (0-100). Default is 55. Grades are rounded: 54.5 rounds up to 55 (pass).",
    help_global_policy: "The global exam replaces the worst grade in the failing category. Everything is recalculated. The grade can go up or down.",
    help_global_weights: "Final = semester grade * W1 + global grade * W2. They should add up to 100 (e.g. 70 + 30).",
    help_global_eligibility: "Minimum grade required to be eligible for the global exam. Leave empty for no restriction (anyone can take it).",

    // Status indicators for rules
    rules_active: "Rules",
    minimum_not_met: "below minimum",
    needs_global: "NEEDS GLOBAL",
    grade_capped_by: "capped by",
    dropped: "dropped",
    eval_below_min: "Below min",
    eval_weights_ok: "Eval weights: 100%",
    eval_weights_warning: "Eval weights: only",
    eval_weights_error: "Eval weights err:",

    // Global exam
    global_exam: "Global Exam",
    global_policy: "Global Exam",
    global_policy_none: "No Global",
    global_policy_weighted: "Weighted",
    global_policy_replaces: "Replaces Worst",
    global_semester_weight: "Semester Weight % (e.g. 70)",
    global_exam_weight: "Global Weight % (e.g. 30)",
    global_min_grade: "Min Grade for Global",
    global_grade: "Global Grade",
    global_needed: "Need in global",
    global_result: "After global",
    global_no_policy: "No global available",
    global_enter_hint: "Press Enter to enter the global exam grade.",

    // Settings
    settings: "Settings",
    settings_nerd_fonts: "Nerd Font Icons",
    settings_nerd_fonts_desc: "Use Nerd Font glyphs (requires a patched font)",
    settings_language: "Language",
    settings_compact_courses: "Compact Courses",
    settings_compact_courses_desc: "Show courses in a single line (name + status only)",
    compact: "Compact View",
    enabled: "Enabled",
    disabled: "Disabled",
    toggle: "Toggle",
    yanked_eval: "Evaluation copied",
    pasted_eval: "Evaluation pasted",
    no_eval_to_yank: "No evaluation to copy",
    no_eval_in_clipboard: "Nothing to paste (copy first with y)",
    bulk_added_evals: "evaluations added",

    // Undo / redo
    undo: "Undo",
    redo: "Redo",
    undo_done: "Undone",
    redo_done: "Redone",
    nothing_to_undo: "Nothing to undo",
    nothing_to_redo: "Nothing to redo",

    // Delete impact
    eval_singular: "evaluation",
    eval_plural: "evaluations",
    category_singular: "category",
    category_plural: "categories",

    // Help overlay
    help_title: "Keyboard Shortcuts",
    help_close_hint: "Esc / Enter to close",
    help_open: "Help",
    help_group_global: "Global",
    help_group_navigation: "Navigation",
    help_group_editing: "Edit",
    help_group_actions: "Actions",
    help_group_view: "View & Settings",
    help_cycle_focus: "Cycle panel",
    help_focus_lr: "Panel left / right",
    help_move_updown: "Move up / down",
    help_close_popup: "Close popup / cancel",
    help_new_generic: "New (course / category / evaluation)",
    help_edit_selected: "Edit selected item",
    help_delete_selected: "Delete selected item",

    // AI import wizard
    tpl_ai: "Create with AI",
    tpl_ai_desc: "Generate from a syllabus PDF/image via ChatGPT, Claude, etc.",
    import_step1_title: "Step 1 of 3 — Copy the AI prompt",
    import_step1_hint: "Copy this prompt and paste it into your AI of choice along with the syllabus PDF or image. The AI will return a JSON blob you'll paste in step 2.",
    import_step1_copy_key: "c: Copy",
    import_step1_copied: "Copied to clipboard",
    import_step1_next: "Enter: Continue",
    import_step1_fullscreen: "f: Fullscreen (mouse-select)",
    import_step1_fullscreen_exit: "Esc / f: Back",
    import_step2_title: "Step 2 of 3 — Paste the AI response",
    import_step2_hint: "Paste here the JSON returned by the AI (Ctrl+V or right-click paste). Code fences are tolerated.",
    import_step2_back: "b / Esc: Back",
    import_step3_title: "Step 3 of 3 — Confirm import",
    import_step3_total_weight: "Total weight",
    import_step3_evaluations: "evaluations",
    import_step3_save_as_template: "t: Also save as template",
    import_step3_confirm: "Enter: Import",
    import_warning_weight_not_100: "Category weights do not sum to 100%",
    import_err_invalid_json: "Invalid JSON",
    import_err_schema_version: "Unsupported schema version:",
    import_err_empty_name: "Course name is required",
    import_err_no_categories: "At least one category is required",
    import_imported_ok: "Course imported",
    import_copy_suffix: "copy",
    import_renamed_to: "Renamed to",
    import_prompt: r#"You are helping a student import a course syllabus into Kalk (a TUI grade tracker).

The user is attaching a PDF or image with the syllabus.  Read it carefully and extract the grading structure.

Respond with ONLY a JSON object — no prose, no markdown code fences, nothing before or after — that matches this schema:

{
  "schema_version": 1,
  "name": "<course name>",
  "passing_grade": <number, default 55>,
  "global_exam": {
    "policy": "<none | weighted | replaces_worst>",
    "semester_weight": <0..1 if policy is weighted, else null>,
    "global_weight":   <0..1 if policy is weighted, else null>,
    "min_grade":       <number or null>
  },
  "categories": [
    {
      "name": "<category name>",
      "weight": <0..100>,
      "drop_lowest": <integer, default 0>,
      "minimum_average": <number or null>,
      "minimum_per_evaluation": <number or null>,
      "weighted_evaluations": <true | false>,
      "averaging_method": "<arithmetic | geometric>",
      "round_before_weighting": <true | false>,
      "evaluations": [
        { "name": "<eval name>", "grade": <number or null>, "weight": <0..100 or null> }
      ]
    }
  ]
}

Rules:
- Output ONLY the JSON.  No code fences, no comments, no explanation.
- Use null for anything not stated in the syllabus.
- If passing_grade is not stated, default to 55.
- Category weights MUST add up to 100.  Adjust if the syllabus is ambiguous.
- global_exam.policy:
  * "none"            — there is no global / final exam.
  * "weighted"        — final_grade = semester * X + global * Y.
  * "replaces_worst"  — the global replaces the worst evaluation of a category.
- Do not invent grades — leave "grade" as null unless the syllabus literally provides it.
- Generate evaluation names like "C1", "C2", "Quiz 1", "Lab 1", "Tarea 1", based on what the syllabus describes.
- If the syllabus lists "drop the lowest N", set drop_lowest accordingly; otherwise 0.

Example:
{"schema_version":1,"name":"Calculus 1","passing_grade":55,"global_exam":{"policy":"weighted","semester_weight":0.7,"global_weight":0.3,"min_grade":null},"categories":[{"name":"Tests","weight":60,"drop_lowest":1,"minimum_average":null,"minimum_per_evaluation":null,"weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"C1","grade":null,"weight":null},{"name":"C2","grade":null,"weight":null},{"name":"C3","grade":null,"weight":null}]},{"name":"Labs","weight":40,"drop_lowest":0,"minimum_average":null,"minimum_per_evaluation":null,"weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"Lab 1","grade":null,"weight":null},{"name":"Lab 2","grade":null,"weight":null},{"name":"Lab 3","grade":null,"weight":null}]}]}
"#,
};

/// Spanish messages.
pub const ES: Messages = Messages {
    // Panel titles
    courses: "Ramos",
    categories: "Categorías",
    evaluations: "Evaluaciones",

    // Course panel
    select_course_to_view: "Selecciona un ramo para ver categorías.",
    select_category_to_view: "Selecciona una categoría para ver evaluaciones.",
    select_course_first: "Selecciona un ramo primero.",
    course_average: "Promedio del Ramo",
    no_grades_yet: "Sin notas aún",
    passed: "APROBADO",
    failed: "REPROBADO",
    current: "Actual",
    need: "Necesitas",
    cannot_pass: "No puedes aprobar",
    no_evaluations: "Sin evaluaciones",

    // Weight validation
    weights_ok: "OK",
    weights_warning: "Solo",
    weights_error: "Err:",
    no_categories: "Sin categorías",

    // Footer / Keybindings
    quit: "Salir",
    new: "Nuevo Ramo",
    new_category: "Nueva Categoría",
    new_eval: "Nueva Evaluación",
    edit: "Editar",
    delete: "Eliminar",
    save_as_template: "Guardar Plantilla",
    balance: "Equilibrar Pesos",
    yank: "Copiar",
    paste: "Pegar",
    bulk_add: "Crear Varias",
    select: "Seleccionar",
    confirm: "Confirmar",
    cancel: "Cancelar",
    next_field: "Siguiente campo",
    change_language: "Idioma",
    enter_global: "Ingresar Nota Global",
    jump_first_last: "Primero / Último",

    // Popups
    select_course_template: "Seleccionar Plantilla",
    new_course: "Nuevo Ramo",
    edit_course: "Editar Ramo",
    new_category_title: "Nueva Categoría",
    edit_category: "Editar Categoría",
    new_evaluation: "Nueva Evaluación",
    edit_evaluation: "Editar Evaluación",
    confirm_delete: "Confirmar Eliminación",
    delete_course_question: "¿Eliminar este ramo?",
    delete_category_question: "¿Eliminar esta categoría?",
    delete_evaluation_question: "¿Eliminar esta evaluación?",
    delete_course_warning: "Se perderán todas las categorías y evaluaciones.",
    delete_category_warning: "Se perderán todas las evaluaciones de esta categoría.",
    delete_evaluation_warning: "Esta acción no se puede deshacer.",
    save_as_template_title: "Guardar como Plantilla",
    delete_template: "Eliminar Plantilla",
    delete_template_question: "¿Eliminar",
    action_cannot_be_undone: "Esta acción no se puede deshacer.",
    select_language: "Seleccionar Idioma",
    bulk_add_title: "Agregar Evaluaciones en Lote",
    bulk_add_hint: "Crea Nombre 1, Nombre 2, ... Nombre N",
    bulk_add_count: "Cantidad",

    // Form labels
    name: "Nombre",
    passing_grade: "Nota de Aprobación (0-100)",
    weight: "Peso % (ej: 80)",
    grade: "Nota (0-100)",
    template_name: "Nombre de Plantilla",
    description: "Descripción",
    current_total: "Total actual",
    will_save_categories: "Guardará categorías de",
    need_to_pass: "Necesitas para Aprobar",
    graded: "calificadas",
    avg: "Prom",

    // Template category names
    tpl_exam: "Certamen",
    tpl_quiz: "Control",
    tpl_homework: "Tarea",
    tpl_lab: "Lab",
    tpl_project: "Proyecto",
    tpl_assessment: "Evaluación",

    // Template names and descriptions
    tpl_exam_quiz_80_20: "Certámenes + Controles (80/20)",
    tpl_exam_quiz_80_20_desc: "3 Certámenes (80%) + 3 Controles (20%)",
    tpl_exam_quiz_70_30: "Certámenes + Controles (70/30)",
    tpl_exam_quiz_70_30_desc: "3 Certámenes (70%) + 3 Controles (30%)",
    tpl_exam_homework_quiz: "Certámenes + Tareas + Controles",
    tpl_exam_homework_quiz_desc: "3 Certámenes (60%) + 4 Tareas (25%) + 3 Controles (15%)",
    tpl_exam_labs: "Certámenes + Labs",
    tpl_exam_labs_desc: "3 Certámenes (70%) + 6 Labs (30%)",
    tpl_project_exams: "Proyecto + Certámenes",
    tpl_project_exams_desc: "Proyecto (40%) + 2 Certámenes (60%)",
    tpl_exams_only: "Solo Certámenes",
    tpl_exams_only_desc: "3 Certámenes (100%)",
    tpl_continuous: "Evaluación Continua",
    tpl_continuous_desc: "10 Evaluaciones semanales (100%)",
    tpl_custom: "Personalizado (Vacío)",
    tpl_custom_desc: "Empieza sin categorías - añade las tuyas",

    // Status messages
    save_error: "Error: No se pudieron guardar los datos",
    save_template_error: "Error: No se pudieron guardar las plantillas",
    config_save_error: "Error: No se pudo guardar la configuración",
    load_error: "Advertencia: No se pudieron cargar datos, iniciando vacío",
    load_template_error: "Advertencia: No se pudieron cargar plantillas",
    config_load_warning: "Advertencia: No se pudo cargar configuración, usando valores por defecto",

    // Misc
    unknown: "Desconocido",
    empty_template_desc: "Plantilla vacía",
    template_suffix: "Plantilla",
    passing: "aprobando",
    below_passing: "bajo aprobación",
    need_grade_impossible: "imposible, máximo es 100",
    need_grade_any: "ya aprobando con cualquier nota",
    need_grade_in_eval: "en esta eval para aprobar",

    // Category rules labels
    advanced_rules: "Reglas Avanzadas",
    advanced_rules_show: "Mostrar reglas",
    advanced_rules_hide: "Ocultar reglas",
    drop_lowest: "Eliminar Peores",
    averaging_method: "Método de Promedio",
    averaging_arithmetic: "Aritmético",
    averaging_geometric: "Geométrico",
    minimum_average: "Promedio Mínimo",
    on_minimum_not_met: "Si No Se Cumple",
    action_final_equals_avg: "Final = Prom. Categoría",
    action_requires_global: "Requiere Global",
    action_fail_course: "Reprobar Ramo",
    minimum_per_evaluation: "Min. Por Eval",
    minimum_one_eval: "Min. Una Eval >=",
    round_before_weighting: "Redondear Categ.",
    weighted_evaluations: "Pesos por Eval",
    eval_weight: "Peso Eval %",
    yes: "Si",
    no: "No",

    // Field help descriptions
    help_toggle: "Ayuda",
    help_name: "Etiqueta de la categoría (ej: Certámenes, Controles, Labs).",
    help_weight: "Cuánto vale esta categoría en la nota final (0-100%).",
    help_drop_lowest: "Descarta las N peores notas antes de promediar. 0 = no eliminar.",
    help_averaging_method: "Aritmético = promedio normal. Geométrico = promedio con raíz del producto (usado en calificación multiplicativa).",
    help_on_min_not_met: "Qué pasa si no se alcanza el promedio mínimo. Final=Prom significa que la nota del ramo es el promedio de esta categoría. Requiere Global significa que debes dar examen global. Reprobar Ramo significa que el ramo se reprueba automáticamente.",
    help_minimum_average: "Nota mínima promedio requerida en esta categoría (ej: 55). Dejar vacío si no aplica.",
    help_min_per_eval: "Nota mínima requerida en cada evaluación individual. Dejar vacío si no aplica.",
    help_min_one_eval: "Al menos una evaluación calificada debe alcanzar esta nota (ej: 50). Si ninguna la alcanza, se activa la acción 'Si No Se Cumple'. Dejar vacío si no aplica.",
    help_round_before_weighting: "Redondea el promedio de la categoria antes de aplicar el peso. Util cuando la universidad redondea por categoria.",
    help_weighted_evaluations: "Cuando esta activado, cada evaluacion tiene su propio peso (deben sumar 100%). Util cuando una categoria tiene sub-componentes con distinto peso (ej. informe 40%, presentacion 60%).",
    help_passing_grade: "Nota minima para aprobar el ramo (0-100). Por defecto es 55. Las notas se redondean: 54.5 sube a 55 (aprueba).",
    help_global_policy: "El global reemplaza la peor nota de la categoría que reprueba. Todo se recalcula. La nota puede subir o bajar.",
    help_global_weights: "Final = nota semestral * P1 + nota global * P2. Deben sumar 100 (ej: 70 + 30).",
    help_global_eligibility: "Nota mínima requerida para poder dar el global. Dejar vacío si no hay restricción (cualquiera puede darlo).",

    // Status indicators for rules
    rules_active: "Reglas",
    minimum_not_met: "bajo mínimo",
    needs_global: "NECESITA GLOBAL",
    grade_capped_by: "limitado por",
    dropped: "eliminada",
    eval_below_min: "Bajo min",
    eval_weights_ok: "Pesos eval: 100%",
    eval_weights_warning: "Pesos eval: solo",
    eval_weights_error: "Pesos eval err:",

    // Global exam
    global_exam: "Examen Global",
    global_policy: "Examen Global",
    global_policy_none: "Sin Global",
    global_policy_weighted: "Ponderado",
    global_policy_replaces: "Reemplaza Peor Nota",
    global_semester_weight: "Peso Semestral % (ej: 70)",
    global_exam_weight: "Peso Global % (ej: 30)",
    global_min_grade: "Nota Min. para Global",
    global_grade: "Nota Global",
    global_needed: "Necesitas en global",
    global_result: "Despues del global",
    global_no_policy: "Sin global disponible",
    global_enter_hint: "Presiona Enter para ingresar la nota del examen global.",

    // Settings
    settings: "Ajustes",
    settings_nerd_fonts: "Iconos Nerd Font",
    settings_nerd_fonts_desc: "Usar glifos Nerd Font (requiere fuente parcheada)",
    settings_language: "Idioma",
    settings_compact_courses: "Ramos Compactos",
    settings_compact_courses_desc: "Mostrar ramos en una sola linea (nombre + estado)",
    compact: "Vista Compacta",
    enabled: "Activado",
    disabled: "Desactivado",
    toggle: "Alternar",
    yanked_eval: "Evaluación copiada",
    pasted_eval: "Evaluación pegada",
    no_eval_to_yank: "No hay evaluación para copiar",
    no_eval_in_clipboard: "Nada para pegar (copia primero con y)",
    bulk_added_evals: "evaluaciones agregadas",

    // Undo / redo
    undo: "Deshacer",
    redo: "Rehacer",
    undo_done: "Deshecho",
    redo_done: "Rehecho",
    nothing_to_undo: "Nada que deshacer",
    nothing_to_redo: "Nada que rehacer",

    // Delete impact
    eval_singular: "evaluación",
    eval_plural: "evaluaciones",
    category_singular: "categoría",
    category_plural: "categorías",

    // Help overlay
    help_title: "Atajos de Teclado",
    help_close_hint: "Esc / Enter para cerrar",
    help_open: "Ayuda",
    help_group_global: "Global",
    help_group_navigation: "Navegación",
    help_group_editing: "Edición",
    help_group_actions: "Acciones",
    help_group_view: "Vista y Ajustes",
    help_cycle_focus: "Cambiar panel",
    help_focus_lr: "Panel izq. / der.",
    help_move_updown: "Mover arriba / abajo",
    help_close_popup: "Cerrar popup / cancelar",
    help_new_generic: "Nuevo (ramo / categoría / evaluación)",
    help_edit_selected: "Editar selección",
    help_delete_selected: "Eliminar selección",

    // AI import wizard
    tpl_ai: "Crear con IA",
    tpl_ai_desc: "Genera el ramo desde un PDF/imagen del programa vía ChatGPT, Claude, etc.",
    import_step1_title: "Paso 1 de 3 — Copiar el prompt para la IA",
    import_step1_hint: "Copia este prompt y pégalo en la IA junto al PDF o imagen del programa. La IA devolverá un JSON que pegarás en el paso 2.",
    import_step1_copy_key: "c: Copiar",
    import_step1_copied: "Copiado al portapapeles",
    import_step1_next: "Enter: Continuar",
    import_step1_fullscreen: "f: Pantalla completa (selección con mouse)",
    import_step1_fullscreen_exit: "Esc / f: Volver",
    import_step2_title: "Paso 2 de 3 — Pegar la respuesta de la IA",
    import_step2_hint: "Pega aquí el JSON devuelto por la IA (Ctrl+V o clic derecho → pegar). Se tolera el formato con ``` ```.",
    import_step2_back: "b / Esc: Atrás",
    import_step3_title: "Paso 3 de 3 — Confirmar importación",
    import_step3_total_weight: "Peso total",
    import_step3_evaluations: "evaluaciones",
    import_step3_save_as_template: "t: Guardar también como plantilla",
    import_step3_confirm: "Enter: Importar",
    import_warning_weight_not_100: "Los pesos de las categorías no suman 100%",
    import_err_invalid_json: "JSON inválido",
    import_err_schema_version: "Versión de schema no soportada:",
    import_err_empty_name: "El nombre del ramo es obligatorio",
    import_err_no_categories: "Se requiere al menos una categoría",
    import_imported_ok: "Ramo importado",
    import_copy_suffix: "copia",
    import_renamed_to: "Renombrado a",
    import_prompt: r#"Estás ayudando a un estudiante a importar el programa de un ramo a Kalk (un gestor de notas en TUI).

El usuario adjuntará un PDF o imagen con el programa del ramo. Léelo con cuidado y extrae la estructura de evaluación.

Responde EXCLUSIVAMENTE con un objeto JSON — sin texto adicional, sin code fences de markdown, nada antes ni después — que cumpla este schema:

{
  "schema_version": 1,
  "name": "<nombre del ramo>",
  "passing_grade": <número, default 55>,
  "global_exam": {
    "policy": "<none | weighted | replaces_worst>",
    "semester_weight": <0..1 si policy es weighted, sino null>,
    "global_weight":   <0..1 si policy es weighted, sino null>,
    "min_grade":       <número o null>
  },
  "categories": [
    {
      "name": "<nombre de la categoría>",
      "weight": <0..100>,
      "drop_lowest": <entero, default 0>,
      "minimum_average": <número o null>,
      "minimum_per_evaluation": <número o null>,
      "weighted_evaluations": <true | false>,
      "averaging_method": "<arithmetic | geometric>",
      "round_before_weighting": <true | false>,
      "evaluations": [
        { "name": "<nombre eval>", "grade": <número o null>, "weight": <0..100 o null> }
      ]
    }
  ]
}

Reglas:
- Devuelve SOLO el JSON. Sin code fences, sin comentarios, sin explicaciones.
- Usa null para cualquier dato que no esté en el programa.
- Si no se menciona passing_grade, usa 55.
- Los pesos de las categorías DEBEN sumar 100. Ajusta si el programa es ambiguo.
- global_exam.policy:
  * "none"            — no hay examen global / final.
  * "weighted"        — nota_final = semestral * X + global * Y.
  * "replaces_worst"  — el global reemplaza la peor nota de una categoría.
- No inventes notas — deja "grade" en null salvo que el programa las dé explícitamente.
- Genera nombres de evaluación tipo "C1", "C2", "Tarea 1", "Lab 1", "Quiz 1" según lo que indique el programa.
- Si el programa permite descartar las N peores notas, pon drop_lowest acorde; sino 0.

Ejemplo:
{"schema_version":1,"name":"Sistemas Digitales","passing_grade":55,"global_exam":{"policy":"weighted","semester_weight":0.7,"global_weight":0.3,"min_grade":null},"categories":[{"name":"Controles","weight":60,"drop_lowest":1,"minimum_average":null,"minimum_per_evaluation":null,"weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"C1","grade":null,"weight":null},{"name":"C2","grade":null,"weight":null},{"name":"C3","grade":null,"weight":null}]},{"name":"Laboratorios","weight":40,"drop_lowest":0,"minimum_average":null,"minimum_per_evaluation":null,"weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"Lab 1","grade":null,"weight":null},{"name":"Lab 2","grade":null,"weight":null},{"name":"Lab 3","grade":null,"weight":null}]}]}
"#,
};

impl Language {
    /// Get the messages for this language.
    pub fn messages(&self) -> &'static Messages {
        match self {
            Language::English => &EN,
            Language::Spanish => &ES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_default_is_english() {
        assert_eq!(Language::default(), Language::English);
    }

    #[test]
    fn test_language_display_names() {
        assert_eq!(Language::English.display_name(), "English");
        assert_eq!(Language::Spanish.display_name(), "Español");
    }

    #[test]
    fn test_all_languages() {
        let all = Language::all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&Language::English));
        assert!(all.contains(&Language::Spanish));
    }
}
