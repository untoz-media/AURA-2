use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionClass {
    Read,
    Act,
    Modify,
    Destructive,
    Sensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionDecision {
    Allow,
    Ask,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionPolicy {
    pub read: PermissionDecision,
    pub act: PermissionDecision,
    pub modify: PermissionDecision,
    pub destructive: PermissionDecision,
    pub sensitive: PermissionDecision,
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

    pub fn set(
        &mut self,
        class: PermissionClass,
        decision: PermissionDecision,
    ) -> Result<(), &'static str> {
        if matches!(
            (class, decision),
            (
                PermissionClass::Sensitive | PermissionClass::Destructive,
                PermissionDecision::Allow
            )
        ) {
            return Err(
                "Sensitive and destructive permissions cannot be permanently allowed.",
            );
        }

        match class {
            PermissionClass::Read => self.read = decision,
            PermissionClass::Act => self.act = decision,
            PermissionClass::Modify => self.modify = decision,
            PermissionClass::Destructive => self.destructive = decision,
            PermissionClass::Sensitive => self.sensitive = decision,
        }

        Ok(())
    }

    pub fn sanitized(mut self) -> Self {
        if self.destructive == PermissionDecision::Allow {
            self.destructive = PermissionDecision::Ask;
        }
        if self.sensitive == PermissionDecision::Allow {
            self.sensitive = PermissionDecision::Ask;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_allows_reversible_actions() {
        let policy = PermissionPolicy::default();
        assert_eq!(
            policy.decision_for(PermissionClass::Act),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn default_policy_requires_confirmation_for_destructive_actions() {
        let policy = PermissionPolicy::default();
        assert_eq!(
            policy.decision_for(PermissionClass::Destructive),
            PermissionDecision::Ask
        );
    }

    #[test]
    fn modify_can_be_changed_to_never() {
        let mut policy = PermissionPolicy::default();
        policy
            .set(PermissionClass::Modify, PermissionDecision::Never)
            .unwrap();
        assert_eq!(policy.modify, PermissionDecision::Never);
    }

    #[test]
    fn destructive_cannot_be_permanently_allowed() {
        let mut policy = PermissionPolicy::default();
        assert!(policy
            .set(PermissionClass::Destructive, PermissionDecision::Allow)
            .is_err());
        assert_eq!(policy.destructive, PermissionDecision::Ask);
    }

    #[test]
    fn unsafe_persisted_values_are_sanitized() {
        let policy = PermissionPolicy {
            sensitive: PermissionDecision::Allow,
            destructive: PermissionDecision::Allow,
            ..PermissionPolicy::default()
        }
        .sanitized();

        assert_eq!(policy.sensitive, PermissionDecision::Ask);
        assert_eq!(policy.destructive, PermissionDecision::Ask);
    }
}
