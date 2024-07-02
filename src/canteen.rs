use std::str::FromStr;

use const_format::concatcp;
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Hash)]
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

const POST_URL_BASE: &str = "https://www.studierendenwerk-pb.de/gastronomie/speiseplaene/";

impl Canteen {
    pub fn get_url(&self) -> &str {
        match self {
            Self::Forum => concatcp!(POST_URL_BASE, "forum/"),
            Self::Academica => concatcp!(POST_URL_BASE, "mensa-academica/"),
            Self::Picknick => concatcp!(POST_URL_BASE, "picknick/"),
            Self::BonaVista => concatcp!(POST_URL_BASE, "bona-vista/"),
            Self::GrillCafe => concatcp!(POST_URL_BASE, "grillcafe/"),
            Self::ZM2 => concatcp!(POST_URL_BASE, "mensa-zm2/"),
            Self::Basilica => concatcp!(POST_URL_BASE, "mensa-basilica-hamm/"),
            Self::Atrium => concatcp!(POST_URL_BASE, "mensa-atrium-lippstadt/"),
        }
    }

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
            invalid => Err(format!("Invalid canteen identifier: {}", invalid)),
        }
    }
}
