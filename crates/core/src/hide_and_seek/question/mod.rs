use crate::{
    hide_and_seek::question::{
        context::QuestionContext, matching::MatchingQuestion, measuring::MeasuringQuestion,
        radar::RadarQuestion, tentacle::TentacleQuestion, thermometer::ThermometerQuestion,
    },
    shape::Shape,
};

pub mod context;
pub mod matching;
pub mod measuring;
pub mod radar;
pub mod tentacle;
pub mod thermometer;

pub enum AnyQuestion {
    Matching(MatchingQuestion),
    Measuring(MeasuringQuestion),
    Thermometer(ThermometerQuestion),
    Radar(RadarQuestion),
    Tentacle(TentacleQuestion),
    // Photo(PhotoQuestion),
}

pub enum ShapeErrorClass {
    /// The shape cannot be computed. Even with complete data this shape is not representable.
    /// For example, a photo question (even with perfect data) cannot be represented as a shape.
    Uncomputable,

    /// The shape cannot be computed due to missing data in the context.
    /// For example, if a question requires POI data that is not downloaded.
    /// This error *should always* be recoverable by providing more data.
    MissingData,

    /// The shape cannot be computed due to lack of entropy in the answer.
    /// For example, some 'null' answers do not constrain the shape at all.
    NoEntropy,

    /// Some questions may have invalid parameters that prevent shape computation.
    /// For example, a thermometer question where the start and end points are identical would not have an angle,
    /// making it impossible to define the great circle.
    InvalidParameters,
}

pub struct ShapeError {
    pub message: String,
    pub resolution_hint: Option<String>,
    pub class: ShapeErrorClass,
}

impl ShapeError {
    pub fn missing_data(nice_name: &str) -> Self {
        Self {
            message: format!("Missing {} Data!", nice_name),
            resolution_hint: Some(format!(
                "Download the '{}' data bundle to visualize this question.",
                nice_name
            )),
            class: ShapeErrorClass::MissingData,
        }
    }
}

pub trait Question {
    type Answer;

    fn to_any(self) -> AnyQuestion;
    fn to_shape(
        self,
        answer: Self::Answer,
        context: Box<dyn QuestionContext>,
    ) -> Result<Box<dyn Shape>, ShapeError>;
}

// the questions are:
// 1. Is your nearest <FIELD: CATEGORY> the same as my nearest <FIELD: CATEGORY>?
// 2. Compared to me are you closer or further from <FIELD: CATEGORY>?
// 3. I've just traveled <FIELD: DISTANCE>. Am I hotter or colder?
// 4. Are you within <FIELD: DISTANCE> of me?
// 5. Of all the <FIELD: CATEGORY> within <FIELD: DISTANCE> of me, which are you closest to?
// 6. Send a photo of <FIELD: SUBJECT>?
