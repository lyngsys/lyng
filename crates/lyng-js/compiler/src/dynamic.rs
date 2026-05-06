use lyng_js_ast::{
    Ast, Decl, Expr, ForInOfLeft, ForInit, FunctionId, ParsedScript, Pattern, Stmt, StmtId,
    VariableKind,
};
use lyng_js_bytecode::CompiledScriptUnit;
use lyng_js_common::{AtomId, AtomTable, SourceId};
use lyng_js_parser::{parse_script, parse_script_with_initial_strict};
use lyng_js_sema::{
    analyze_direct_eval_script, analyze_script, DirectEvalScriptAnalysisOptions, ScriptSema,
};
use lyng_js_types::RealmRef;

use crate::compile_script;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DynamicFunctionKind {
    Ordinary,
    Generator,
    Async,
    AsyncGenerator,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DynamicFunctionCacheKey {
    realm: RealmRef,
    kind: DynamicFunctionKind,
    parameters_source: Box<str>,
    body_source: Box<str>,
    strict_caller: bool,
}

impl DynamicFunctionCacheKey {
    #[inline]
    pub fn new(
        realm: RealmRef,
        kind: DynamicFunctionKind,
        parameters_source: &str,
        body_source: &str,
        strict_caller: bool,
    ) -> Self {
        Self {
            realm,
            kind,
            parameters_source: parameters_source.into(),
            body_source: body_source.into(),
            strict_caller,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DynamicCompilationStage {
    Parse,
    Semantic,
    Compile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicCompilationError {
    stage: DynamicCompilationStage,
}

impl DynamicCompilationError {
    #[inline]
    pub const fn new(stage: DynamicCompilationStage) -> Self {
        Self { stage }
    }

    #[inline]
    pub const fn stage(&self) -> &DynamicCompilationStage {
        &self.stage
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DynamicScriptAnalysisMode {
    Script,
    InitialStrict(bool),
    DynamicFunction,
    DirectEval {
        initial_strict: bool,
        options: DirectEvalScriptAnalysisOptions,
    },
}

pub struct DynamicScriptAnalysis {
    parsed: ParsedScript,
    sema: ScriptSema,
}

impl DynamicScriptAnalysis {
    #[inline]
    pub const fn parsed(&self) -> &ParsedScript {
        &self.parsed
    }

    #[inline]
    pub const fn sema(&self) -> &ScriptSema {
        &self.sema
    }

    #[inline]
    pub fn sema_mut(&mut self) -> &mut ScriptSema {
        &mut self.sema
    }

    pub fn root_var_initializer_names(&self) -> Vec<AtomId> {
        let ast = &self.parsed.ast;
        let body = ast.get_script(self.parsed.root).body;
        let mut names = Vec::new();
        for &stmt in ast.get_stmt_list(body) {
            collect_var_initializer_names_from_stmt(ast, stmt, &mut names);
        }
        names
    }
}

fn push_unique_atom(names: &mut Vec<AtomId>, name: AtomId) {
    if !names.contains(&name) {
        names.push(name);
    }
}

fn collect_var_initializer_names_from_pattern(
    ast: &Ast,
    pattern: lyng_js_ast::PatternId,
    names: &mut Vec<AtomId>,
) {
    match ast.get_pattern(pattern) {
        Pattern::Identifier { name, .. } => push_unique_atom(names, *name),
        Pattern::Object {
            properties, rest, ..
        } => {
            for property in ast.get_obj_pattern_prop_list(*properties) {
                collect_var_initializer_names_from_pattern(ast, property.value, names);
            }
            if let Some(rest) = rest {
                collect_var_initializer_names_from_pattern(ast, *rest, names);
            }
        }
        Pattern::Array { elements, rest, .. } => {
            for element in ast.get_opt_pattern_elem_list(*elements).iter().flatten() {
                collect_var_initializer_names_from_pattern(ast, element.pattern, names);
            }
            if let Some(rest) = rest {
                collect_var_initializer_names_from_pattern(ast, *rest, names);
            }
        }
        Pattern::Assignment { left, .. } => {
            collect_var_initializer_names_from_pattern(ast, *left, names);
        }
        Pattern::InvalidPattern { .. } => {}
    }
}

fn collect_var_initializer_names_from_decl(
    ast: &Ast,
    decl: lyng_js_ast::DeclId,
    names: &mut Vec<AtomId>,
) {
    let Decl::Variable {
        kind: VariableKind::Var,
        declarators,
        ..
    } = ast.get_decl(decl)
    else {
        return;
    };
    for declarator in ast.get_var_declarator_list(*declarators) {
        if declarator.init.is_some() {
            collect_var_initializer_names_from_pattern(ast, declarator.id, names);
        }
    }
}

fn collect_var_initializer_names_from_stmt(ast: &Ast, stmt: StmtId, names: &mut Vec<AtomId>) {
    match ast.get_stmt(stmt) {
        Stmt::Block { body, .. } => {
            for &stmt in ast.get_stmt_list(*body) {
                collect_var_initializer_names_from_stmt(ast, stmt, names);
            }
        }
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            collect_var_initializer_names_from_stmt(ast, *consequent, names);
            if let Some(alternate) = alternate {
                collect_var_initializer_names_from_stmt(ast, *alternate, names);
            }
        }
        Stmt::DoWhile { body, .. } | Stmt::While { body, .. } | Stmt::With { body, .. } => {
            collect_var_initializer_names_from_stmt(ast, *body, names);
        }
        Stmt::For { init, body, .. } => {
            if let Some(ForInit::Declaration(decl)) = init {
                collect_var_initializer_names_from_decl(ast, *decl, names);
            }
            collect_var_initializer_names_from_stmt(ast, *body, names);
        }
        Stmt::ForIn { left, body, .. } | Stmt::ForOf { left, body, .. } => {
            if let ForInOfLeft::Declaration(decl) = left {
                collect_var_initializer_names_from_decl(ast, *decl, names);
            }
            collect_var_initializer_names_from_stmt(ast, *body, names);
        }
        Stmt::Switch { cases, .. } => {
            for case in ast.get_switch_case_list(*cases) {
                for &stmt in ast.get_stmt_list(case.consequent) {
                    collect_var_initializer_names_from_stmt(ast, stmt, names);
                }
            }
        }
        Stmt::Labeled { body, .. } => collect_var_initializer_names_from_stmt(ast, *body, names),
        Stmt::Try {
            block,
            handler,
            finalizer,
            ..
        } => {
            collect_var_initializer_names_from_stmt(ast, *block, names);
            if let Some(handler) = handler {
                collect_var_initializer_names_from_stmt(ast, handler.body, names);
            }
            if let Some(finalizer) = finalizer {
                collect_var_initializer_names_from_stmt(ast, *finalizer, names);
            }
        }
        Stmt::Declaration { decl, .. } => {
            collect_var_initializer_names_from_decl(ast, *decl, names)
        }
        Stmt::Empty { .. }
        | Stmt::Expression { .. }
        | Stmt::Continue { .. }
        | Stmt::Break { .. }
        | Stmt::Return { .. }
        | Stmt::Throw { .. }
        | Stmt::Debugger { .. }
        | Stmt::InvalidStatement { .. } => {}
    }
}

pub struct DynamicScriptCompilation {
    analysis: DynamicScriptAnalysis,
    unit: CompiledScriptUnit,
}

impl DynamicScriptCompilation {
    #[inline]
    pub const fn analysis(&self) -> &DynamicScriptAnalysis {
        &self.analysis
    }

    #[inline]
    pub const fn unit(&self) -> &CompiledScriptUnit {
        &self.unit
    }

    #[inline]
    pub fn into_unit(self) -> CompiledScriptUnit {
        self.unit
    }
}

pub fn dynamic_function_source(
    parameters_source: &str,
    body_source: &str,
    kind: DynamicFunctionKind,
) -> String {
    match kind {
        DynamicFunctionKind::Ordinary => {
            format!("(function anonymous({parameters_source}\n) {{\n{body_source}\n}})")
        }
        DynamicFunctionKind::Generator => {
            format!("(function* anonymous({parameters_source}\n) {{\n{body_source}\n}})")
        }
        DynamicFunctionKind::Async => {
            format!("(async function anonymous({parameters_source}\n) {{\n{body_source}\n}})")
        }
        DynamicFunctionKind::AsyncGenerator => {
            format!("(async function* anonymous({parameters_source}\n) {{\n{body_source}\n}})")
        }
    }
}

fn dynamic_function_parameter_validation_source(
    parameters_source: &str,
    kind: DynamicFunctionKind,
) -> String {
    match kind {
        DynamicFunctionKind::Ordinary => {
            format!("(function anonymous({parameters_source}\n) {{}})")
        }
        DynamicFunctionKind::Generator => {
            format!("(function* anonymous({parameters_source}\n) {{}})")
        }
        DynamicFunctionKind::Async => {
            format!("(async function anonymous({parameters_source}\n) {{}})")
        }
        DynamicFunctionKind::AsyncGenerator => {
            format!("(async function* anonymous({parameters_source}\n) {{}})")
        }
    }
}

pub fn analyze_dynamic_script(
    atoms: &mut AtomTable,
    source_id: SourceId,
    source_text: &str,
    mode: DynamicScriptAnalysisMode,
) -> Result<DynamicScriptAnalysis, DynamicCompilationError> {
    let analysis = analyze_dynamic_script_with_diagnostics(atoms, source_id, source_text, mode)?;
    if analysis.sema.diagnostics.has_errors() {
        return Err(DynamicCompilationError::new(
            DynamicCompilationStage::Semantic,
        ));
    }
    Ok(analysis)
}

pub fn analyze_dynamic_script_with_diagnostics(
    atoms: &mut AtomTable,
    source_id: SourceId,
    source_text: &str,
    mode: DynamicScriptAnalysisMode,
) -> Result<DynamicScriptAnalysis, DynamicCompilationError> {
    let parsed = match mode {
        DynamicScriptAnalysisMode::Script | DynamicScriptAnalysisMode::DynamicFunction => {
            parse_script(atoms, source_id, source_text)
        }
        DynamicScriptAnalysisMode::InitialStrict(initial_strict)
        | DynamicScriptAnalysisMode::DirectEval { initial_strict, .. } => {
            parse_script_with_initial_strict(atoms, source_id, source_text, initial_strict)
        }
    };
    if parsed.diagnostics.has_errors() {
        return Err(DynamicCompilationError::new(DynamicCompilationStage::Parse));
    }

    let sema = match mode {
        DynamicScriptAnalysisMode::DirectEval { options, .. } => {
            analyze_direct_eval_script(&parsed, atoms, options)
        }
        DynamicScriptAnalysisMode::DynamicFunction => {
            let options = dynamic_function_wrapper_function(&parsed).map_or_else(
                DirectEvalScriptAnalysisOptions::new,
                |function| {
                    DirectEvalScriptAnalysisOptions::new()
                        .with_suppressed_function_name_bindings(vec![function])
                },
            );
            analyze_direct_eval_script(&parsed, atoms, options)
        }
        DynamicScriptAnalysisMode::Script | DynamicScriptAnalysisMode::InitialStrict(_) => {
            analyze_script(&parsed, atoms)
        }
    };

    Ok(DynamicScriptAnalysis { parsed, sema })
}

fn dynamic_function_wrapper_function(parsed: &ParsedScript) -> Option<FunctionId> {
    let script = parsed.ast.get_script(parsed.root);
    let [stmt] = parsed.ast.get_stmt_list(script.body) else {
        return None;
    };
    let Stmt::Expression { expression, .. } = parsed.ast.get_stmt(*stmt) else {
        return None;
    };

    let mut expr = *expression;
    while let Expr::ParenthesizedExpression {
        expression: inner, ..
    } = parsed.ast.get_expr(expr)
    {
        expr = *inner;
    }
    let Expr::FunctionExpression { function, .. } = parsed.ast.get_expr(expr) else {
        return None;
    };
    Some(*function)
}

pub fn compile_analyzed_dynamic_script(
    analysis: &DynamicScriptAnalysis,
    atoms: &mut AtomTable,
) -> Result<CompiledScriptUnit, DynamicCompilationError> {
    compile_script(analysis.parsed(), analysis.sema(), atoms)
        .map_err(|_| DynamicCompilationError::new(DynamicCompilationStage::Compile))
}

pub fn compile_dynamic_script_source(
    atoms: &mut AtomTable,
    source_id: SourceId,
    source_text: &str,
    mode: DynamicScriptAnalysisMode,
) -> Result<DynamicScriptCompilation, DynamicCompilationError> {
    let analysis = analyze_dynamic_script(atoms, source_id, source_text, mode)?;
    let unit = compile_analyzed_dynamic_script(&analysis, atoms)?;
    Ok(DynamicScriptCompilation { analysis, unit })
}

pub fn compile_dynamic_function(
    atoms: &mut AtomTable,
    source_id: SourceId,
    parameters_source: &str,
    body_source: &str,
    kind: DynamicFunctionKind,
) -> Result<DynamicScriptCompilation, DynamicCompilationError> {
    let parameters_text = dynamic_function_parameter_validation_source(parameters_source, kind);
    if parse_script(atoms, source_id, &parameters_text)
        .diagnostics
        .has_errors()
    {
        return Err(DynamicCompilationError::new(DynamicCompilationStage::Parse));
    }

    let source_text = dynamic_function_source(parameters_source, body_source, kind);
    compile_dynamic_script_source(
        atoms,
        source_id,
        &source_text,
        DynamicScriptAnalysisMode::DynamicFunction,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_function_source_wraps_goal_by_kind() {
        assert_eq!(
            dynamic_function_source("a", "return a;", DynamicFunctionKind::Ordinary),
            "(function anonymous(a\n) {\nreturn a;\n})"
        );
        assert_eq!(
            dynamic_function_source("a", "yield a;", DynamicFunctionKind::Generator),
            "(function* anonymous(a\n) {\nyield a;\n})"
        );
        assert_eq!(
            dynamic_function_source("a", "return a;", DynamicFunctionKind::Async),
            "(async function anonymous(a\n) {\nreturn a;\n})"
        );
        assert_eq!(
            dynamic_function_source("a", "yield a;", DynamicFunctionKind::AsyncGenerator),
            "(async function* anonymous(a\n) {\nyield a;\n})"
        );
    }

    #[test]
    fn compile_dynamic_script_source_reports_parse_stage() {
        let mut atoms = AtomTable::new();
        let result = compile_dynamic_script_source(
            &mut atoms,
            SourceId::new(1),
            "let =",
            DynamicScriptAnalysisMode::Script,
        );
        assert_eq!(
            result.err().map(|error| error.stage().clone()),
            Some(DynamicCompilationStage::Parse)
        );
    }

    #[test]
    fn dynamic_function_parameter_validation_allows_annex_b_html_comments() {
        let mut atoms = AtomTable::new();

        let open = compile_dynamic_function(
            &mut atoms,
            SourceId::new(1),
            "<!--",
            "",
            DynamicFunctionKind::Ordinary,
        );
        assert_eq!(open.err().map(|error| error.stage().clone()), None);

        let close = compile_dynamic_function(
            &mut atoms,
            SourceId::new(2),
            "\n-->",
            "",
            DynamicFunctionKind::Ordinary,
        );
        assert_eq!(close.err().map(|error| error.stage().clone()), None);
    }
}
