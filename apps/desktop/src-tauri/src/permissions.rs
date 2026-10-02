#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionClass {
    Read,
    Act,
    Modify,
    Destructive,
    Sensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Ask,
    Block,
}

#[derive(Debug, Clone)]
pub struct PermissionPolicy {
    read: PermissionDecision,
    act: PermissionDecision,
    modify: PermissionDecision,
    destructive: PermissionDecision,
    sensitive: PermissionDecision,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self {
            read: PermissionDecision::Allow,
            act: PermissionDecision::Allow,
            modify: PermissionDecision::Ask,
            destructive: PermissionDecision::Ask,
            sensitive: PermissionDecision::Ask,
        }
    }
}

impl PermissionPolicy {
    pub fn decision_for(&self, class: PermissionClass) -> PermissionDecision {
        match class {
            PermissionClass::Read => self.read,
            PermissionClass::Act => self.act,
            PermissionClass::Modify => self.modify,
            PermissionClass::Destructive => self.destructive,
            PermissionClass::Sensitive => self.sensitive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_allows_reversible_actions() {
        let policy = PermissionPolicy::default();
        assert_eq!(policy.decision_for(PermissionClass::Act), PermissionDecision::Allow);
    }

    #[test]
    fn default_policy_requires_confirmation_for_destructive_actions() {
        let policy = PermissionPolicy::default();
        assert_eq!(
            policy.decision_for(PermissionClass::Destructive),
            PermissionDecision::Ask
        );
    }
}
