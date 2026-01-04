use crate::shape::{
    Shape,
    builtin::circle::Circle,
    compiler::{Register, SdfCompiler},
    types::{Centimeters, Position},
};

pub struct RadarQuestion {
    pub center: Position,
    pub radius: Centimeters,
}

pub enum RadarQuestionAnswer {
    Hit,
    Miss,
}

pub struct RadarQuestionShape {
    pub question: RadarQuestion,
    pub answer: RadarQuestionAnswer,
    pub context: Box<dyn QuestionContext>,
}

impl Shape for RadarQuestionShape {
    fn build_into(&self, compiler: &mut SdfCompiler) -> Register {
        let result = compiler.with(&Circle::new(self.question.center, self.question.radius));

        match self.answer {
            RadarQuestionAnswer::Hit => result,
            RadarQuestionAnswer::Miss => compiler.invert(result),
        }
    }
}
