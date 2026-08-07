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

//! Spanish translation table.

use super::Messages;

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
    minimum_one_eval: "Min. Una Eval ≥",
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
  "credits": <entero o null>,
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
      "minimum_one_eval": <número o null>,
      "on_minimum_not_met":          "<final_equals_average | requires_global | fail_course>",
      "on_min_per_eval_not_met":     "<final_equals_average | requires_global | fail_course>",
      "on_min_one_eval_not_met":     "<final_equals_average | requires_global | fail_course>",
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
- "credits" son los créditos del ramo (SCT, ECTS, o lo que use la institución).
  Usa null si el programa no los indica — no los inventes.
- Reglas de mínimos, y qué pasa cuando no se cumplen:
  * "minimum_average"        — la categoría debe promediar al menos esto.
  * "minimum_per_evaluation" — CADA evaluación debe alcanzar al menos esto.
  * "minimum_one_eval"       — AL MENOS UNA evaluación debe alcanzar esto.
  Cada uno tiene su "on_..._not_met" que le dice a Kalk qué significa fallarlo:
  * "final_equals_average" — la nota final pasa a ser el promedio de esa categoría.
  * "requires_global"      — el estudiante debe rendir el examen global.
  * "fail_course"          — el ramo se reprueba directamente.
  Pon la acción sólo cuando el programa declare un mínimo; si no, usa
  "final_equals_average".  Lee con cuidado: "debe promediar 50 para aprobar el
  ramo" es fail_course, mientras que "debe promediar 50 o va a global" es
  requires_global.  Equivocarse acá cambia si al estudiante se le dice que aprobó.

Ejemplo:
{"schema_version":1,"name":"Calculo 1","passing_grade":55,"credits":5,"global_exam":{"policy":"weighted","semester_weight":0.7,"global_weight":0.3,"min_grade":null},"categories":[{"name":"Controles","weight":60,"drop_lowest":1,"minimum_average":null,"minimum_per_evaluation":null,"minimum_one_eval":null,"on_minimum_not_met":"final_equals_average","on_min_per_eval_not_met":"final_equals_average","on_min_one_eval_not_met":"final_equals_average","weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"C1","grade":null,"weight":null},{"name":"C2","grade":null,"weight":null},{"name":"C3","grade":null,"weight":null}]},{"name":"Laboratorios","weight":40,"drop_lowest":0,"minimum_average":null,"minimum_per_evaluation":null,"minimum_one_eval":null,"on_minimum_not_met":"final_equals_average","on_min_per_eval_not_met":"final_equals_average","on_min_one_eval_not_met":"final_equals_average","weighted_evaluations":false,"averaging_method":"arithmetic","round_before_weighting":false,"evaluations":[{"name":"Lab 1","grade":null,"weight":null},{"name":"Lab 2","grade":null,"weight":null},{"name":"Lab 3","grade":null,"weight":null}]}]}
"#,

    // Semesters
    semester_name_prefix: "Semestre",
    semester_manage: "Nuevo / Renombrar / Eliminar",
    home_open: "Semestres (desde Ramos)",
    metric_status: "Estado",
    metric_progress: "Avance",
    metric_evaluations: "evaluaciones",
    metric_trend: "Tendencia",
    metric_courses_breakdown: "Ramos",
    credits_label: "Créditos (opcional)",
    course_singular: "ramo",
    course_plural: "ramos",
    semesters: "Semestres",
    semester_new: "Nuevo Semestre",
    semester_rename: "Renombrar",
    semester_delete: "Eliminar Semestre",
    semester_move: "Mover",
    semester_open: "Abrir",
    semester_cannot_delete_last: "No se puede eliminar el único semestre.",
    semester_delete_warning: "Esto elimina el semestre y todo lo que contiene.",
    semester_name_label: "Nombre del semestre",
    home_no_courses: "Aún no hay ramos en este semestre.",
    metric_average: "Promedio",
    metric_average_weighted: "por créditos",
    metric_passed: "Aprobando",
    metric_failing: "Reprobando",
    metric_pending_global: "Global pendiente",
    metric_credits: "Créditos",
    metric_credits_at_risk: "En riesgo",
    metric_critical: "Más crítico",
    metric_cumulative: "Acumulado",
    metric_no_data: "Sin datos",
};
