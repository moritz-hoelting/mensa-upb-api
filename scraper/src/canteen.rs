use const_format::concatcp;
use shared::Canteen;

const POST_URL_BASE: &str = "https://www.studierendenwerk-pb.de/gastronomie/speiseplaene/";

pub trait CanteenExt {
    fn get_url(&self) -> &str;
}

impl CanteenExt for Canteen {
    fn get_url(&self) -> &str {
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
}
