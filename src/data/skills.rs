#[derive(Debug, Clone, Default)]
pub struct PassiveSkill {
    pub name: Option<String>,
    pub is_notable: bool,
    pub stats: Vec<super::stats::Stat>,
}
