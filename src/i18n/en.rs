// Kalk — your academic dashboard in the terminal.
// Copyright (C) 2026  yfloress
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

//! English translation table.

use super::Messages;

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
    minimum_one_eval: "Min. One Eval ≥",
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
  "credits": <integer or null>,
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
      "minimum_one_eval": <number or null>,
      "on_minimum_not_met":          "<final_equals_average | requires_global | fail_course>",
      "on_min_per_eval_not_met":     "<final_equals_average | requires_global | fail_course>",
      "on_min_one_eval_not_met":     "<final_equals_average | requires_global | fail_course>",
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
- "credits" is the course's credit value (SCT, ECTS, or whatever the institution
  uses).  Use null when the syllabus does not state it — do not guess.
- Minimum rules, and what happens when one is not met:
  * "minimum_average"        — the category must average at least this to pass.
  * "minimum_per_evaluation" — EVERY evaluation must reach at least this.
  * "minimum_one_eval"       — AT LEAST ONE evaluation must reach this.
  Each has a matching "on_..._not_met" telling Kalk what a failure means:
  * "final_equals_average" — the final grade becomes that category's average.
  * "requires_global"      — the student must sit the global exam.
  * "fail_course"          — the course is failed outright.
  Set the action only when the syllabus states a minimum; otherwise use
  "final_equals_average".  Read the wording carefully: "must average 50 to pass
  the course" is fail_course, while "must average 50 or go to the global" is
  requires_global.  Getting this wrong changes whether the student is told they
  passed.

Example:
{"schema_version":1,"name":"Calculus 1","passing_grade":55,"credits":5,"global_exam":{"policy":"weighted","semester_weight":0.7,"global_weight":0.3,"min_grade":null},"categories":[{"name":"Tests","weight":60,"drop_lowest":1,"minimum_average":null,"minimum_per_evaluation":null,"minimum_one_eval":null,"on_minimum_not_met":"final_equals_average","on_min_per_eval_not_met":"final_equals_average","on_min_one_eval_not_met":"final_equals_average","weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"C1","grade":null,"weight":null},{"name":"C2","grade":null,"weight":null},{"name":"C3","grade":null,"weight":null}]},{"name":"Labs","weight":40,"drop_lowest":0,"minimum_average":null,"minimum_per_evaluation":null,"minimum_one_eval":null,"on_minimum_not_met":"final_equals_average","on_min_per_eval_not_met":"final_equals_average","on_min_one_eval_not_met":"final_equals_average","weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"Lab 1","grade":null,"weight":null},{"name":"Lab 2","grade":null,"weight":null},{"name":"Lab 3","grade":null,"weight":null}]}]}
"#,

    // Semesters
    semester_name_prefix: "Semester",
    metric_in_play: "In play",
    metric_minimums_unmet: "Minimums unmet",
    metric_unrecoverable: "Beyond saving",
    metric_needs: "needs",
    metric_in: "in",
    metric_lost: "lost",
    semester_manage: "New / Rename / Delete",
    home_open: "Semesters (from Courses)",
    metric_status: "Status",
    metric_progress: "Progress",
    metric_evaluations: "evaluations",
    metric_trend: "Trend",
    metric_courses_breakdown: "Courses",
    credits_label: "Credits (optional)",
    course_singular: "course",
    course_plural: "courses",
    semesters: "Semesters",
    semester_new: "New Semester",
    semester_rename: "Rename",
    semester_delete: "Delete Semester",
    semester_move: "Move",
    semester_open: "Open",
    semester_cannot_delete_last: "Cannot delete the only semester.",
    semester_delete_warning: "This deletes the semester and everything in it.",
    semester_name_label: "Semester name",
    home_no_courses: "No courses in this semester yet.",
    metric_average: "Average",
    metric_average_weighted: "by credits",
    metric_passed: "Passing",
    metric_failing: "Failing",
    metric_pending_global: "Pending global",
    metric_credits: "Credits",
    metric_credits_at_risk: "At risk",
    metric_critical: "Most critical",
    metric_cumulative: "Cumulative",
    metric_no_data: "No data",
};
