use shared::Canteen;

#[extend::ext]
pub impl Canteen {
    fn get_venue_id(&self) -> &'static str {
        match self {
            Self::Academica => "mensa",
            Self::Forum => "mensa-forum",
            Self::ZM2 => "mensa-zm2",
            Self::Basilica => "mensa-hamm",
            Self::Atrium => "mensa-lippstadt",
            Self::GrillCafe => "grill-cafe",
        }
    }
}
