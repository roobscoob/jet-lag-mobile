use std::collections::HashMap;

use naga::{Expression, Function, LocalVariable, Module, ScalarKind, Span, Statement, TypeInner};

use crate::shape::{compiled::shader::routine::RoutineResult, compiler::Register};

pub fn compile_subtract(
    left: Register,
    right: Register,
) -> impl Fn(
    &mut Module,
    naga::Handle<Function>,
    &HashMap<Register, naga::Handle<naga::LocalVariable>>,
    &str,
) -> RoutineResult {
    move |module, into, register_map, unique_id| {
        let left_var = *register_map.get(&left).expect("Left register not found");
        let right_var = *register_map.get(&right).expect("Right register not found");

        // Create LocalVariable pointer expressions BEFORE emit_start
        // (LocalVariable expressions are already in scope and must not be emitted)
        let left_ptr = module.functions[into]
            .expressions
            .append(Expression::LocalVariable(left_var), Span::UNDEFINED);
        let right_ptr = module.functions[into]
            .expressions
            .append(Expression::LocalVariable(right_var), Span::UNDEFINED);

        // Track emit start - only Load/Unary/Math expressions will be created after this
        let emit_start = module.functions[into].expressions.len();

        // Load left
        let left_expr = module.functions[into]
            .expressions
            .append(Expression::Load { pointer: left_ptr }, Span::UNDEFINED);

        // Load right
        let right_expr = module.functions[into]
            .expressions
            .append(Expression::Load { pointer: right_ptr }, Span::UNDEFINED);

        // Negate right
        let neg_right = module.functions[into].expressions.append(
            Expression::Unary {
                op: naga::UnaryOperator::Negate,
                expr: right_expr,
            },
            Span::UNDEFINED,
        );

        // max(left, -right)
        let subtract_expr = module.functions[into].expressions.append(
            Expression::Math {
                fun: naga::MathFunction::Max,
                arg: left_expr,
                arg1: Some(neg_right),
                arg2: None,
                arg3: None,
            },
            Span::UNDEFINED,
        );

        // Emit all the value expressions (Load, Negate, Max)
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
                name: Some(format!("{}__subtract_distance", unique_id)),
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
                value: subtract_expr,
            },
            Span::UNDEFINED,
        );

        RoutineResult {
            argument_len: 0,
            variable: result_var,
        }
    }
}
