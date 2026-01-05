pub trait StateType: Clone + Send + Sync + 'static + PartialEq + std::fmt::Debug {
    fn can_transition_to(&self, target: &Self) -> bool;
}
