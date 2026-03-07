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
    pub select: &'static str,
    pub confirm: &'static str,
    pub cancel: &'static str,
    pub next_field: &'static str,
    pub change_language: &'static str,

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
    pub minimum_per_evaluation: &'static str,
    pub round_before_weighting: &'static str,
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
    pub help_round_before_weighting: &'static str,

    // Status indicators for rules
    pub rules_active: &'static str,
    pub minimum_not_met: &'static str,
    pub needs_global: &'static str,
    pub grade_capped_by: &'static str,
    pub dropped: &'static str,
    pub eval_below_min: &'static str,
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
    new: "New",
    new_category: "New Category",
    new_eval: "New Eval",
    edit: "Edit",
    delete: "Delete",
    save_as_template: "Save as Template",
    balance: "Balance",
    select: "Select",
    confirm: "Confirm",
    cancel: "Cancel",
    next_field: "Next field",
    change_language: "Language",

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
    advanced_rules_show: "Shift+A: Show rules",
    advanced_rules_hide: "Shift+A: Hide rules",
    drop_lowest: "Drop Lowest",
    averaging_method: "Average Method",
    averaging_arithmetic: "Arithmetic",
    averaging_geometric: "Geometric",
    minimum_average: "Min. Average",
    on_minimum_not_met: "If Not Met",
    action_final_equals_avg: "Final = Category Avg",
    action_requires_global: "Requires Global",
    minimum_per_evaluation: "Min. Per Eval",
    round_before_weighting: "Round Category",
    yes: "Yes",
    no: "No",

    // Field help descriptions
    help_toggle: "?: Help",
    help_name: "Category label (e.g. Exams, Quizzes, Labs).",
    help_weight: "How much this category counts toward the final grade (0-100%).",
    help_drop_lowest: "Discard the N worst grades before averaging. 0 = keep all.",
    help_averaging_method: "Arithmetic = normal average. Geometric = root-product average (useful for multiplicative grading).",
    help_on_min_not_met: "What happens when the minimum average is not reached. Final=Avg means the course grade equals this category average. Requires Global means you must take a global exam.",
    help_minimum_average: "Minimum average required in this category (e.g. 55). Leave empty for none.",
    help_min_per_eval: "Minimum grade required on each individual evaluation. Leave empty for none.",
    help_round_before_weighting: "Round the category average before applying weight. Useful when the university rounds per category.",

    // Status indicators for rules
    rules_active: "Rules",
    minimum_not_met: "below minimum",
    needs_global: "NEEDS GLOBAL",
    grade_capped_by: "capped by",
    dropped: "dropped",
    eval_below_min: "below min",
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
    new: "Nuevo",
    new_category: "Nueva Categoría",
    new_eval: "Nueva Eval",
    edit: "Editar",
    delete: "Eliminar",
    save_as_template: "Guardar como Plantilla",
    balance: "Balancear",
    select: "Seleccionar",
    confirm: "Confirmar",
    cancel: "Cancelar",
    next_field: "Siguiente campo",
    change_language: "Idioma",

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
    advanced_rules_show: "Shift+A: Mostrar reglas",
    advanced_rules_hide: "Shift+A: Ocultar reglas",
    drop_lowest: "Eliminar Peores",
    averaging_method: "Método de Promedio",
    averaging_arithmetic: "Aritmético",
    averaging_geometric: "Geométrico",
    minimum_average: "Promedio Mínimo",
    on_minimum_not_met: "Si No Se Cumple",
    action_final_equals_avg: "Final = Prom. Categoría",
    action_requires_global: "Requiere Global",
    minimum_per_evaluation: "Min. Por Eval",
    round_before_weighting: "Redondear Categoría",
    yes: "Sí",
    no: "No",

    // Field help descriptions
    help_toggle: "?: Ayuda",
    help_name: "Etiqueta de la categoría (ej: Certámenes, Controles, Labs).",
    help_weight: "Cuánto vale esta categoría en la nota final (0-100%).",
    help_drop_lowest: "Descarta las N peores notas antes de promediar. 0 = no eliminar.",
    help_averaging_method: "Aritmético = promedio normal. Geométrico = promedio con raíz del producto (usado en calificación multiplicativa).",
    help_on_min_not_met: "Qué pasa si no se alcanza el promedio mínimo. Final=Prom significa que la nota del ramo es el promedio de esta categoría. Requiere Global significa que debes dar examen global.",
    help_minimum_average: "Nota mínima promedio requerida en esta categoría (ej: 55). Dejar vacío si no aplica.",
    help_min_per_eval: "Nota mínima requerida en cada evaluación individual. Dejar vacío si no aplica.",
    help_round_before_weighting: "Redondea el promedio de la categoría antes de ponderar. Útil cuando la universidad redondea por categoría.",

    // Status indicators for rules
    rules_active: "Reglas",
    minimum_not_met: "bajo mínimo",
    needs_global: "NECESITA GLOBAL",
    grade_capped_by: "limitado por",
    dropped: "eliminada",
    eval_below_min: "bajo min",
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
