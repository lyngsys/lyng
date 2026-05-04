use lyng_js_ast::{Expr, FunctionId, ParsedScript, Stmt};
use lyng_js_bytecode::CompiledScriptUnit;
use lyng_js_common::{AtomTable, SourceId};
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
            format!("(function anonymous({parameters_source}) {{}})")
        }
        DynamicFunctionKind::Generator => {
            format!("(function* anonymous({parameters_source}) {{}})")
        }
        DynamicFunctionKind::Async => {
            format!("(async function anonymous({parameters_source}) {{}})")
        }
        DynamicFunctionKind::AsyncGenerator => {
            format!("(async function* anonymous({parameters_source}) {{}})")
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
}
