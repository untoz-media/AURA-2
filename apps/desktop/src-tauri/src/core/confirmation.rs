use crate::permissions::PermissionClass;

#[derive(Clone, Debug)]
pub(crate) struct PendingConfirmation {
    pub command: String,
    pub source: String,
    pub permission: PermissionClass,
    pub expires_at_ms: u64,
}

pub(crate) const CONFIRMATION_TTL_MS: u64 = 60_000;

pub(crate) fn validate_pending_confirmation(
    pending: &PendingConfirmation,
    command: &str,
    source: &str,
    now_ms: u64,
) -> Result<PermissionClass, &'static str> {
    if pending.expires_at_ms <= now_ms {
        return Err("Confirmation expired. Submit the command again.");
    }

    if pending.command != command || pending.source != source {
        return Err("Confirmation does not match this command.");
    }

    Ok(pending.permission)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending() -> PendingConfirmation {
        PendingConfirmation {
            command: "Close OBS".to_string(),
            source: "overlay".to_string(),
            permission: PermissionClass::Modify,
            expires_at_ms: 60_000,
        }
    }

    #[test]
    fn exact_confirmation_is_valid() {
        assert_eq!(
            validate_pending_confirmation(&pending(), "Close OBS", "overlay", 59_999),
            Ok(PermissionClass::Modify)
        );
    }

    #[test]
    fn expired_confirmation_is_rejected() {
        assert_eq!(
            validate_pending_confirmation(&pending(), "Close OBS", "overlay", 60_000),
            Err("Confirmation expired. Submit the command again.")
        );
    }

    #[test]
    fn command_mismatch_is_rejected() {
        assert_eq!(
            validate_pending_confirmation(&pending(), "Shutdown PC", "overlay", 10),
            Err("Confirmation does not match this command.")
        );
    }

    #[test]
    fn source_mismatch_is_rejected() {
        assert_eq!(
            validate_pending_confirmation(&pending(), "Close OBS", "desktop", 10),
            Err("Confirmation does not match this command.")
        );
    }
}
