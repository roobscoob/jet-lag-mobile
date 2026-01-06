use crate::{
    hide_and_seek::question::{Question, context::QuestionContext},
    shape::{
        Shape,
        builtin::circle::Circle,
        compiler::{Register, SdfCompiler},
        types::Centimeters,
    },
};

pub struct RadarQuestion {
    pub center: geo::Point,
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

impl Question for RadarQuestion {
    type Answer = RadarQuestionAnswer;

    fn to_any(self) -> super::AnyQuestion {
        super::AnyQuestion::Radar(self)
    }

    fn to_shape(
        self,
        answer: Self::Answer,
        context: Box<dyn QuestionContext>,
    ) -> Result<Box<dyn Shape>, super::ShapeError> {
        Ok(Box::new(RadarQuestionShape {
            question: self,
            answer,
            context,
        }))
    }
}
