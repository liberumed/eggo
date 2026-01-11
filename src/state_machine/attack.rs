#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AttackPhase {
    #[default]
    WindUp,
    Strike,
    Recovery,
}
