use crate::{
    hide_and_seek::question::{Question, context::QuestionContext},
    shape::{
        Shape,
        compiler::{Register, SdfCompiler},
        types::Centimeters,
    },
};

pub enum MeasuringTarget {
    CommercialAirport,
    HighSpeedRailLine,
    RailStation,
    InternationalBorder,
    FirstAdministrativeDivisionBorder,
    SecondAdministrativeDivisionBorder,
    SeaLevel,
    BodyOfWater,
    Coastline,
    Mountain,
    Park,
    AmusementPark,
    Zoo,
    Aquarium,
    GolfCourse,
    Museum,
    MovieTheater,
    Hospital,
    Library,
    ForeignConsulate,
}

pub struct MeasuringQuestion {
    pub category: MeasuringTarget,

    // sometimes this is altitude for MeasuringTarget::SeaLevel because Ben and Adam hate me personally.
    pub distance: Centimeters,
}

pub enum MeasuringQuestionAnswer {
    Null,
    Closer,
    Further,
}

pub struct MeasuringQuestionShape {
    pub question: MeasuringQuestion,
    pub answer: MeasuringQuestionAnswer,
    pub context: Box<dyn QuestionContext>,
}

impl Shape for MeasuringQuestionShape {
    fn build_into(&self, compiler: &mut SdfCompiler) -> Register {
        let vdf = match self.question.category {
            // atrociously special-cased.
            MeasuringTarget::SeaLevel => {
                let contour = compiler.with_contour_texture(
                    self.context.sea_level_contour_texture().unwrap(),
                    self.question.distance,
                );

                // if they answer they're "further" from sea level, then their elevation is *greater*
                // therefore: the hider area needs to be negative where the elevation is greater than the zero_value.

                return match self.answer {
                    MeasuringQuestionAnswer::Null => {
                        unreachable!("should be filtered by shape generation")
                    }
                    MeasuringQuestionAnswer::Closer => contour,
                    MeasuringQuestionAnswer::Further => compiler.invert(contour),
                };
            }

            // also special-cased.
            MeasuringTarget::HighSpeedRailLine => {
                let paths = self
                    .context
                    .high_speed_rail_lines()
                    .unwrap()
                    .iter()
                    .map(|path| compiler.geodesic_string(path.positions.clone()))
                    .collect::<Vec<_>>();

                compiler.union(paths)
            }

            MeasuringTarget::CommercialAirport => compiler.point_cloud(
                self.context
                    .get_all_pois("airport")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::RailStation => compiler.point_cloud(
                self.context
                    .transit_context()
                    .all_complexes()
                    .iter()
                    .map(|c| c.center())
                    .collect(),
            ),

            MeasuringTarget::InternationalBorder => {
                let shape = compiler.with_vdg(
                    self.context
                        .get_all_areas_as_vdg("international_border")
                        .unwrap(),
                );

                compiler.edge(shape)
            }

            MeasuringTarget::FirstAdministrativeDivisionBorder => {
                let shape = compiler.with_vdg(
                    self.context
                        .get_all_areas_as_vdg("first_administrative_division")
                        .unwrap(),
                );

                compiler.edge(shape)
            }

            MeasuringTarget::SecondAdministrativeDivisionBorder => {
                let shape = compiler.with_vdg(
                    self.context
                        .get_all_areas_as_vdg("second_administrative_division")
                        .unwrap(),
                );

                compiler.edge(shape)
            }

            MeasuringTarget::BodyOfWater => {
                let shape =
                    compiler.with_vdg(self.context.get_all_areas_as_vdg("water_body").unwrap());

                compiler.edge(shape)
            }

            MeasuringTarget::Coastline => {
                let shape =
                    compiler.with_vdg(self.context.get_all_areas_as_vdg("landmass").unwrap());

                compiler.edge(shape)
            }

            MeasuringTarget::Mountain => compiler.point_cloud(
                self.context
                    .get_all_pois("mountain")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::Park => compiler.point_cloud(
                self.context
                    .get_all_pois("park")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::AmusementPark => compiler.point_cloud(
                self.context
                    .get_all_pois("amusement_park")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::Zoo => compiler.point_cloud(
                self.context
                    .get_all_pois("zoo")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::Aquarium => compiler.point_cloud(
                self.context
                    .get_all_pois("aquarium")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::GolfCourse => compiler.point_cloud(
                self.context
                    .get_all_pois("golf_course")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::Museum => compiler.point_cloud(
                self.context
                    .get_all_pois("museum")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::MovieTheater => compiler.point_cloud(
                self.context
                    .get_all_pois("movie_theater")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::Hospital => compiler.point_cloud(
                self.context
                    .get_all_pois("hospital")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::Library => compiler.point_cloud(
                self.context
                    .get_all_pois("library")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),

            MeasuringTarget::ForeignConsulate => compiler.point_cloud(
                self.context
                    .get_all_pois("foreign_consulate")
                    .unwrap()
                    .iter()
                    .map(|a| a.position)
                    .collect(),
            ),
        };

        let dilated = compiler.dilate(vdf, self.question.distance);

        match self.answer {
            MeasuringQuestionAnswer::Null => {
                unreachable!("should be filtered by shape generation")
            }
            MeasuringQuestionAnswer::Closer => dilated,
            MeasuringQuestionAnswer::Further => compiler.invert(dilated),
        }
    }
}

impl Question for MeasuringQuestion {
    type Answer = MeasuringQuestionAnswer;

    fn to_any(self) -> super::AnyQuestion {
        super::AnyQuestion::Measuring(self)
    }

    fn to_shape(
        self,
        answer: Self::Answer,
        context: Box<dyn QuestionContext>,
    ) -> Result<Box<dyn Shape>, super::ShapeError> {
        if matches!(answer, MeasuringQuestionAnswer::Null) {
            return Err(super::ShapeError {
                message: "No POIs available to answer Measuring Question.".to_string(),
                resolution_hint: Some(
                    "Your game map should include POIs for this category.".to_string(),
                ),
                class: super::ShapeErrorClass::NoEntropy,
            });
        }

        match self.category {
            MeasuringTarget::RailStation => {}

            MeasuringTarget::CommercialAirport => {
                if !context.has_poi_category("airport") {
                    return Err(super::ShapeError::missing_data("Airports"));
                }
            }

            MeasuringTarget::HighSpeedRailLine => {
                if !context.has_high_speed_rail_line_data() {
                    return Err(super::ShapeError::missing_data("High-Speed Rail Lines"));
                }
            }

            MeasuringTarget::InternationalBorder => {
                if !context.has_area_category("international_border") {
                    return Err(super::ShapeError::missing_data("Administrative Divisions"));
                }
            }

            MeasuringTarget::FirstAdministrativeDivisionBorder => {
                if !context.has_area_category("first_administrative_division") {
                    return Err(super::ShapeError::missing_data("Administrative Divisions"));
                }
            }

            MeasuringTarget::SecondAdministrativeDivisionBorder => {
                if !context.has_area_category("second_administrative_division") {
                    return Err(super::ShapeError::missing_data("Administrative Divisions"));
                }
            }

            MeasuringTarget::SeaLevel => {
                if !context.has_sea_level_contour_texture() {
                    return Err(super::ShapeError::missing_data("Sea Level Contour Texture"));
                }
            }

            MeasuringTarget::BodyOfWater => {
                if !context.has_area_category("water_body") {
                    return Err(super::ShapeError::missing_data("Water Bodies"));
                }
            }

            MeasuringTarget::Coastline => {
                if !context.has_area_category("landmass") {
                    return Err(super::ShapeError::missing_data("Landmasses"));
                }
            }

            MeasuringTarget::Mountain => {
                if !context.has_poi_category("mountain") {
                    return Err(super::ShapeError::missing_data("Mountains"));
                }
            }

            MeasuringTarget::Park => {
                if !context.has_poi_category("park") {
                    return Err(super::ShapeError::missing_data("Parks"));
                }
            }

            MeasuringTarget::AmusementPark => {
                if !context.has_poi_category("amusement_park") {
                    return Err(super::ShapeError::missing_data("Amusement Parks"));
                }
            }

            MeasuringTarget::Zoo => {
                if !context.has_poi_category("zoo") {
                    return Err(super::ShapeError::missing_data("Zoos"));
                }
            }

            MeasuringTarget::Aquarium => {
                if !context.has_poi_category("aquarium") {
                    return Err(super::ShapeError::missing_data("Aquariums"));
                }
            }

            MeasuringTarget::GolfCourse => {
                if !context.has_poi_category("golf_course") {
                    return Err(super::ShapeError::missing_data("Golf Courses"));
                }
            }

            MeasuringTarget::Museum => {
                if !context.has_poi_category("museum") {
                    return Err(super::ShapeError::missing_data("Museums"));
                }
            }

            MeasuringTarget::MovieTheater => {
                if !context.has_poi_category("movie_theater") {
                    return Err(super::ShapeError::missing_data("Movie Theaters"));
                }
            }

            MeasuringTarget::Hospital => {
                if !context.has_poi_category("hospital") {
                    return Err(super::ShapeError::missing_data("Hospitals"));
                }
            }

            MeasuringTarget::Library => {
                if !context.has_poi_category("library") {
                    return Err(super::ShapeError::missing_data("Libraries"));
                }
            }

            MeasuringTarget::ForeignConsulate => {
                if !context.has_poi_category("foreign_consulate") {
                    return Err(super::ShapeError::missing_data("Foreign Consulates"));
                }
            }
        };

        Ok(Box::new(MeasuringQuestionShape {
            question: self,
            answer,
            context,
        }))
    }
}
