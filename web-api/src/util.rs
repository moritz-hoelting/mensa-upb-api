use std::str::FromStr as _;

use shared::Canteen;

pub fn parse_canteens_comma_separated(s: &str) -> Vec<Result<Canteen, String>> {
    s.split(',').map(Canteen::from_str).collect()
}
