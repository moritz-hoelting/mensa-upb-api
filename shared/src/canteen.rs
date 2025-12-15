use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Hash, Serialize, Deserialize,
)]
pub enum Canteen {
    Forum,
    Academica,
    Picknick,
    BonaVista,
    GrillCafe,
    ZM2,
    Basilica,
    Atrium,
}

impl Canteen {
    pub fn get_identifier(&self) -> &str {
        match self {
            Self::Forum => "forum",
            Self::Academica => "academica",
            Self::Picknick => "picknick",
            Self::BonaVista => "bona-vista",
            Self::GrillCafe => "grillcafe",
            Self::ZM2 => "zm2",
            Self::Basilica => "basilica",
            Self::Atrium => "atrium",
        }
    }
}

impl FromStr for Canteen {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "forum" => Ok(Self::Forum),
            "academica" => Ok(Self::Academica),
            "picknick" => Ok(Self::Picknick),
            "bona-vista" => Ok(Self::BonaVista),
            "grillcafe" => Ok(Self::GrillCafe),
            "zm2" => Ok(Self::ZM2),
            "basilica" => Ok(Self::Basilica),
            "atrium" => Ok(Self::Atrium),
            invalid => Err(format!("Invalid canteen identifier: {invalid}")),
        }
    }
}
