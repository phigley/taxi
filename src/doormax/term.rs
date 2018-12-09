#[derive(Debug, Clone, Copy, Enum)]
pub enum Term {
    TouchWallN,
    TouchWallS,
    TouchWallE,
    TouchWallW,
    OnPassenger,
    OnDestination,
    HasPassenger,
}
