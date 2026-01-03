use std::collections::HashMap;

use crate::hide_and_seek::question::field::Field;

pub enum TextSegment {
    Literal(String),
    FieldValue { name: String },
}

pub struct QuestionTemplate {
    pub text: Vec<TextSegment>,
}
