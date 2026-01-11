use std::collections::HashMap;

use naga::{Expression, Function, LocalVariable, Module, ScalarKind, Span, Statement, TypeInner};

use crate::shape::{compiled::shader::routine::RoutineResult, compiler::Register};

pub fn compile_dilate(
    register: Register,
) -> impl Fn(
    &mut Module,
    naga::Handle<Function>,
    &HashMap<Register, naga::Handle<naga::LocalVariable>>,
    &str,
) -> RoutineResult {
    move |module, into, register_map, unique_id| {
        // Find the dilate routine in the module
        let dilate_routine = module
            .functions
            .iter()
            .find(|(_, f)| f.name.as_deref() == Some("dilate"))
            .map(|(handle, _)| handle)
            .expect("dilate routine not found in module");

        // Get the input register variable
        let input_var = *register_map.get(&register).expect("Register not found");

        // Load the input distance variable
        let var_ptr = module.functions[into]
            .expressions
            .append(Expression::LocalVariable(input_var), Span::UNDEFINED);
        let emit_start = module.functions[into].expressions.len();
        let distance_expr = module.functions[into]
            .expressions
            .append(Expression::Load { pointer: var_ptr }, Span::UNDEFINED);

        // Emit the Load expression
        let emit_range = module.functions[into].expressions.range_from(emit_start);
        module.functions[into]
            .body
            .push(Statement::Emit(emit_range), Span::UNDEFINED);

        // Get function arguments (sample, idx_ptr)
        // FunctionArgument expressions are already in scope - no need to emit
        let sample_expr = module.functions[into]
            .expressions
            .append(Expression::FunctionArgument(0), Span::UNDEFINED);

        let idx_ptr_expr = module.functions[into]
            .expressions
            .append(Expression::FunctionArgument(1), Span::UNDEFINED);

        // Find i32 type
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

        // Call dilate routine with (distance, sample, idx_ptr)
        let call_result = module.functions[into]
            .expressions
            .append(Expression::CallResult(dilate_routine), Span::UNDEFINED);

        module.functions[into].body.push(
            Statement::Call {
                function: dilate_routine,
                arguments: vec![distance_expr, sample_expr, idx_ptr_expr],
                result: Some(call_result),
            },
            Span::UNDEFINED,
        );

        // Create local variable for the result
        let result_var = module.functions[into].local_variables.append(
            LocalVariable {
                name: Some(format!("{}__dilate_distance", unique_id)),
                ty: i32_type,
                init: None,
            },
            Span::UNDEFINED,
        );

        let result_ptr = module.functions[into]
            .expressions
            .append(Expression::LocalVariable(result_var), Span::UNDEFINED);

        // Store call result in variable
        module.functions[into].body.push(
            Statement::Store {
                pointer: result_ptr,
                value: call_result,
            },
            Span::UNDEFINED,
        );

        RoutineResult {
            argument_len: 1,
            variable: result_var,
        }
    }
}
