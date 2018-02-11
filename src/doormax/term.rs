#[derive(Debug, Clone, Copy, EnumMap)]
pub enum Term {
    TouchWallN,
    TouchWallS,
    TouchWallE,
    TouchWallW,
    OnPassenger,
    OnDestination,
    HasPassenger,
}
