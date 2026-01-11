use std::collections::HashMap;

use naga::{Expression, Function, LocalVariable, Module, ScalarKind, Span, Statement, TypeInner};

use crate::shape::{compiled::shader::routine::RoutineResult, compiler::Register};

// Edge - absolute value of distance (distance to boundary)
pub fn compile_edge(
    register: Register,
) -> impl Fn(
    &mut Module,
    naga::Handle<Function>,
    &HashMap<Register, naga::Handle<naga::LocalVariable>>,
    &str,
) -> RoutineResult {
    move |module, into, register_map, unique_id| {
        let var = *register_map.get(&register).expect("Register not found");

        // Load the variable
        let var_ptr = module.functions[into]
            .expressions
            .append(Expression::LocalVariable(var), Span::UNDEFINED);
        let emit_start = module.functions[into].expressions.len();
        let load_expr = module.functions[into]
            .expressions
            .append(Expression::Load { pointer: var_ptr }, Span::UNDEFINED);

        // Take absolute value
        let abs_expr = module.functions[into].expressions.append(
            Expression::Math {
                fun: naga::MathFunction::Abs,
                arg: load_expr,
                arg1: None,
                arg2: None,
                arg3: None,
            },
            Span::UNDEFINED,
        );

        // Emit the value expressions
        let emit_range = module.functions[into].expressions.range_from(emit_start);
        module.functions[into]
            .body
            .push(Statement::Emit(emit_range), Span::UNDEFINED);

        let i32_type = module
            .types
            .iter()
            .find(|(_, ty)| {
                matches!(
                    ty.inner,
                    TypeInner::Scalar(naga::Scalar {
                        kind: ScalarKind::Sint,
                        width: 4
                    })
                )
            })
            .map(|(handle, _)| handle)
            .expect("i32 type not found in module");

        let result_var = module.functions[into].local_variables.append(
            LocalVariable {
                name: Some(format!("{}__edge_distance", unique_id)),
                ty: i32_type,
                init: None,
            },
            Span::UNDEFINED,
        );

        let var_ptr = module.functions[into]
            .expressions
            .append(Expression::LocalVariable(result_var), Span::UNDEFINED);

        module.functions[into].body.push(
            Statement::Store {
                pointer: var_ptr,
                value: abs_expr,
            },
            Span::UNDEFINED,
        );

        RoutineResult {
            argument_len: 0,
            variable: result_var,
        }
    }
}
