#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialAccountMatchIdAccess {
    Granted,
    NotMember,
    Unrecognized,
}
