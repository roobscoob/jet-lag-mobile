use std::sync::Arc;

use itertools::Itertools;

use crate::{
    hide_and_seek::question::context::QuestionContext,
    shape::{
        Shape,
        compiler::{Register, SdfCompiler},
        instruction::BoundaryOverlapResolution,
        types::Centimeters,
    },
    transit::TripIdentifier,
};

pub enum TentacleTarget {
    Museum,
    Library,
    MovieTheater,
    Hospital,
    MetroLine,
    Zoo,
    Aquarium,
    AmusementPark,
}

pub struct TentacleQuestion {
    pub center: geo::Point,
    pub radius: Centimeters,
    pub target: TentacleTarget,
}

pub enum TentacleQuestionAnswer {
    OutOfRadius,
    WithinRadius { closest_id: Arc<str> },
}

pub struct TentacleQuestionShape {
    pub question: TentacleQuestion,
    pub answer: TentacleQuestionAnswer,
    pub context: Box<dyn QuestionContext>,
}

impl Shape for TentacleQuestionShape {
    fn build_into(&self, compiler: &mut SdfCompiler) -> Register {
        let TentacleQuestionAnswer::WithinRadius { ref closest_id } = self.answer else {
            let center = compiler.point(self.question.center);
            let circle = compiler.dilate(center, self.question.radius);
            return compiler.invert(circle);
        };

        let (other, tentacle) = match self.question.target {
            // this will require special handling :yipee:
            TentacleTarget::MetroLine => {
                let complexes = self
                    .context
                    .transit_context()
                    .get_trip(&TripIdentifier::new(closest_id))
                    .unwrap()
                    .stop_events()
                    .iter()
                    .map(|e| {
                        self.context
                            .transit_context()
                            .get_station(&e.station_id)
                            .unwrap()
                            .complex()
                    })
                    .unique_by(|v| v.identifier())
                    .collect::<Vec<_>>();

                let other = self
                    .context
                    .transit_context()
                    .all_complexes()
                    .iter()
                    .filter_map(|c| {
                        (!complexes.iter().any(|cc| cc.identifier() == c.identifier()))
                            .then_some(c.center())
                    });

                let question = complexes.iter().map(|c| c.center()).collect::<Vec<_>>();

                let osp = compiler.point_cloud(other.collect());
                let qsp = compiler.point_cloud(question);

                (
                    compiler.dilate(osp, self.context.game_state().seeker_hiding_radius()),
                    compiler.dilate(qsp, self.context.game_state().seeker_hiding_radius()),
                )
            }

            TentacleTarget::Museum => {
                let other = self
                    .context
                    .get_all_pois("museum")
                    .unwrap()
                    .iter()
                    .filter_map(|v| (*v.id != **closest_id).then_some(v.position));

                let question = self
                    .context
                    .get_poi("museum", &**closest_id)
                    .unwrap()
                    .position;

                (
                    compiler.point_cloud(other.collect()),
                    compiler.point(question),
                )
            }

            TentacleTarget::Library => {
                let other = self
                    .context
                    .get_all_pois("library")
                    .unwrap()
                    .iter()
                    .filter_map(|v| (*v.id != **closest_id).then_some(v.position));

                let question = self
                    .context
                    .get_poi("library", &**closest_id)
                    .unwrap()
                    .position;

                (
                    compiler.point_cloud(other.collect()),
                    compiler.point(question),
                )
            }

            TentacleTarget::MovieTheater => {
                let other = self
                    .context
                    .get_all_pois("movie_theater")
                    .unwrap()
                    .iter()
                    .filter_map(|v| (*v.id != **closest_id).then_some(v.position));

                let question = self
                    .context
                    .get_poi("movie_theater", &**closest_id)
                    .unwrap()
                    .position;

                (
                    compiler.point_cloud(other.collect()),
                    compiler.point(question),
                )
            }

            TentacleTarget::Hospital => {
                let other = self
                    .context
                    .get_all_pois("hospital")
                    .unwrap()
                    .iter()
                    .filter_map(|v| (*v.id != **closest_id).then_some(v.position));

                let question = self
                    .context
                    .get_poi("hospital", &**closest_id)
                    .unwrap()
                    .position;

                (
                    compiler.point_cloud(other.collect()),
                    compiler.point(question),
                )
            }

            TentacleTarget::Zoo => {
                let other = self
                    .context
                    .get_all_pois("zoo")
                    .unwrap()
                    .iter()
                    .filter_map(|v| (*v.id != **closest_id).then_some(v.position));

                let question = self.context.get_poi("zoo", &**closest_id).unwrap().position;

                (
                    compiler.point_cloud(other.collect()),
                    compiler.point(question),
                )
            }

            TentacleTarget::Aquarium => {
                let other = self
                    .context
                    .get_all_pois("aquarium")
                    .unwrap()
                    .iter()
                    .filter_map(|v| (*v.id != **closest_id).then_some(v.position));

                let question = self
                    .context
                    .get_poi("aquarium", &**closest_id)
                    .unwrap()
                    .position;

                (
                    compiler.point_cloud(other.collect()),
                    compiler.point(question),
                )
            }

            TentacleTarget::AmusementPark => {
                let other = self
                    .context
                    .get_all_pois("amusement_park")
                    .unwrap()
                    .iter()
                    .filter_map(|v| (*v.id != **closest_id).then_some(v.position));

                let question = self
                    .context
                    .get_poi("amusement_park", &**closest_id)
                    .unwrap()
                    .position;

                (
                    compiler.point_cloud(other.collect()),
                    compiler.point(question),
                )
            }
        };

        compiler.boundary(tentacle, other, BoundaryOverlapResolution::Inside)
    }
}
