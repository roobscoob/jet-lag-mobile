#define_import_path template::instruction::dilate

#import template::arguments::{
    popArgument,
    argument_read_i32
}

// Instruction: Dilate
fn dilate(distance: i32, sample: vec2<f32>, idx_ptr: ptr<function, u32>) -> i32 {
    let argument = popArgument(idx_ptr);
    let radius_cm = argument_read_i32(argument, 0u);
    
    return distance - radius_cm;
}