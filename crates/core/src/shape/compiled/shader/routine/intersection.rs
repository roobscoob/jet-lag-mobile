use std::collections::HashMap;

use naga::{Expression, Function, LocalVariable, Module, ScalarKind, Span, Statement, TypeInner};

use crate::shape::{compiled::shader::routine::RoutineResult, compiler::Register};

pub fn compile_intersection(
    registers: Vec<Register>,
) -> impl Fn(
    &mut Module,
    naga::Handle<Function>,
    &HashMap<Register, naga::Handle<naga::LocalVariable>>,
    &str,
) -> RoutineResult {
    move |module, into, register_map, unique_id| {
        let var_handles: Vec<_> = registers
            .iter()
            .map(|reg| *register_map.get(reg).expect("Register not found in map"))
            .collect();

        // Pre-create all LocalVariable pointer expressions BEFORE emit_start
        // (LocalVariable expressions are already in scope and must not be emitted)
        let var_ptrs: Vec<_> = var_handles
            .iter()
            .map(|&var| {
                module.functions[into]
                    .expressions
                    .append(Expression::LocalVariable(var), Span::UNDEFINED)
            })
            .collect();

        // Track emit start - only Load and Math expressions will be created after this
        let emit_start = module.functions[into].expressions.len();

        fn build_max_tree(
            module: &mut Module,
            into: naga::Handle<Function>,
            ptrs: &[naga::Handle<naga::Expression>],
        ) -> naga::Handle<naga::Expression> {
            match ptrs.len() {
                0 => panic!("Cannot intersect zero values"),
                1 => {
                    // Load from the pointer
                    module.functions[into]
                        .expressions
                        .append(Expression::Load { pointer: ptrs[0] }, Span::UNDEFINED)
                }
                _ => {
                    let mid = ptrs.len() / 2;
                    let left = build_max_tree(module, into, &ptrs[..mid]);
                    let right = build_max_tree(module, into, &ptrs[mid..]);

                    module.functions[into].expressions.append(
                        Expression::Math {
                            fun: naga::MathFunction::Max,
                            arg: left,
                            arg1: Some(right),
                            arg2: None,
                            arg3: None,
                        },
                        Span::UNDEFINED,
                    )
                }
            }
        }

        let max_expr = build_max_tree(module, into, &var_ptrs);

        // Emit all the value expressions (Load and Max operations)
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
                name: Some(format!("{}__intersection_distance", unique_id)),
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
                value: max_expr,
            },
            Span::UNDEFINED,
        );

        RoutineResult {
            argument_len: 0,
            variable: result_var,
        }
    }
}
