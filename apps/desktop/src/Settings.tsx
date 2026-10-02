import type { ReactNode } from "react";
import type {
  AppStatus,
  PermissionClass,
  PermissionDecision,
  PermissionPolicy,
  RuntimeState,
} from "./bridge/types";
import { SectionLabel, ShortcutKey, Surface } from "./design-system/components";

type SettingsSection =
  | "general"
  | "appearance"
  | "privacy"
  | "permissions"
  | "models"
  | "voice"
  | "overlay"
  | "shortcuts"
  | "integrations";

type Props = {
  activeSection: SettingsSection;
  onSectionChange: (section: SettingsSection) => void;
  appStatus: AppStatus | null;
  runtimeState: RuntimeState;
  onPausedChange: (paused: boolean) => Promise<RuntimeState>;
  onBackgroundChange: (backgroundEnabled: boolean) => Promise<RuntimeState>;
  onAutostartChange: (autostartEnabled: boolean) => Promise<RuntimeState>;
  permissionPolicy: PermissionPolicy;
  onPermissionChange: (
    permissionClass: PermissionClass,
    decision: PermissionDecision,
  ) => Promise<PermissionPolicy>;
  onResetPermissions: () => Promise<PermissionPolicy>;
};

const sections: Array<{
  id: SettingsSection;
  label: string;
  icon: string;
}> = [
  { id: "general", label: "General", icon: "⌂" },
  { id: "appearance", label: "Appearance", icon: "◐" },
  { id: "privacy", label: "Privacy", icon: "◇" },
  { id: "permissions", label: "Permissions", icon: "✓" },
  { id: "models", label: "Models", icon: "◎" },
  { id: "voice", label: "Voice", icon: "∿" },
  { id: "overlay", label: "Overlay", icon: "▱" },
  { id: "shortcuts", label: "Shortcuts", icon: "⌘" },
  { id: "integrations", label: "Integrations", icon: "⌁" },
];

function SettingRow({
  title,
  description,
  trailing,
}: {
  title: string;
  description: string;
  trailing: ReactNode;
}) {
  return (
    <div className="setting-row">
      <div className="setting-copy">
        <strong>{title}</strong>
        <span>{description}</span>
      </div>
      <div className="setting-trailing">{trailing}</div>
    </div>
  );
}

function Toggle({
  checked,
  onChange,
  disabled = false,
  label,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  label: string;
}) {
  return (
    <button
      type="button"
      className={`settings-toggle ${checked ? "checked" : ""}`}
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span />
    </button>
  );
}

function PermissionSelect({
  value,
  onChange,
  allowPermanent = true,
  label,
}: {
  value: PermissionDecision;
  onChange: (decision: PermissionDecision) => void;
  allowPermanent?: boolean;
  label: string;
}) {
  return (
    <select
      className="permission-select"
      value={value}
      aria-label={label}
      onChange={(event) => onChange(event.target.value as PermissionDecision)}
    >
      {allowPermanent && <option value="allow">Allow</option>}
      <option value="ask">Ask</option>
      <option value="never">Never</option>
    </select>
  );
}

function Badge({
  children,
  tone = "neutral",
}: {
  children: ReactNode;
  tone?: "neutral" | "ready" | "planned" | "warning";
}) {
  return <span className={`settings-badge ${tone}`}>{children}</span>;
}

export default function Settings({
  activeSection,
  onSectionChange,
  appStatus,
  runtimeState,
  onPausedChange,
  onBackgroundChange,
  onAutostartChange,
  permissionPolicy,
  onPermissionChange,
  onResetPermissions,
}: Props) {
  return (
    <section className="settings-layout">
      <aside className="settings-nav" aria-label="Settings sections">
        <div className="settings-nav-heading">Settings</div>
        {sections.map((section) => (
          <button
            type="button"
            key={section.id}
            className={`settings-nav-item ${activeSection === section.id ? "active" : ""}`}
            onClick={() => onSectionChange(section.id)}
          >
            <span aria-hidden="true">{section.icon}</span>
            {section.label}
          </button>
        ))}
      </aside>

      <div className="settings-content">
        {activeSection === "general" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>General</h2>
              <p>Core behaviour and desktop lifecycle.</p>
            </header>

            <Surface className="settings-card">
              <SectionLabel>Runtime</SectionLabel>
              <SettingRow
                title="Pause AURA"
                description="Stops new commands and future background actions until resumed."
                trailing={
                  <Toggle
                    checked={runtimeState.paused}
                    onChange={(value) => void onPausedChange(value)}
                    label="Pause AURA"
                  />
                }
              />
              <SettingRow
                title="Run in background"
                description="Keep AURA, the tray and the Overlay available after closing the main window."
                trailing={
                  <Toggle
                    checked={runtimeState.backgroundEnabled}
                    onChange={(value) => void onBackgroundChange(value)}
                    label="Run AURA in background"
                  />
                }
              />
              <SettingRow
                title="Start with Windows"
                description="Launch AURA silently in the background when you sign in to Windows."
                trailing={
                  <Toggle
                    checked={runtimeState.autostartEnabled}
                    onChange={(value) => void onAutostartChange(value)}
                    label="Start AURA with Windows"
                  />
                }
              />
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>About</SectionLabel>
              <SettingRow
                title={appStatus?.name ?? "AURA-2"}
                description={appStatus?.stage ?? "Desktop Foundation"}
                trailing={<Badge>{appStatus?.version ?? "0.2.0"}</Badge>}
              />
              <SettingRow
                title="Execution model"
                description="AURA keeps local execution as the default."
                trailing={<Badge tone="ready">Local-first</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "appearance" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Appearance</h2>
              <p>Visual behaviour for the desktop app and Overlay.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Theme</SectionLabel>
              <SettingRow
                title="AURA Dark"
                description="Current AURA-2 visual system."
                trailing={<Badge tone="ready">Default</Badge>}
              />
              <SettingRow
                title="Reduced motion"
                description="AURA automatically respects the Windows/browser reduced-motion preference."
                trailing={<Badge tone="ready">System</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "privacy" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Privacy</h2>
              <p>Control what AURA can access and where processing happens.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Processing</SectionLabel>
              <SettingRow
                title="Local-first processing"
                description="Prefer local tools and models whenever the requested capability supports it."
                trailing={<Badge tone="ready">Enabled</Badge>}
              />
              <SettingRow
                title="Cloud assistance"
                description="Optional cloud routing will require explicit configuration."
                trailing={<Badge tone="planned">Not configured</Badge>}
              />
              <SettingRow
                title="Screen access"
                description="Vision access will remain permission-based."
                trailing={<Badge tone="planned">M007</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "permissions" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Permissions</h2>
              <p>Rules that will determine what AURA may do on your computer.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel trailing={<Badge tone="ready">Active</Badge>}>
                Action levels
              </SectionLabel>
              <SettingRow
                title="Read"
                description="Inspect allowed local and system information."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.read}
                    onChange={(decision) => void onPermissionChange("read", decision)}
                    label="Read permission"
                  />
                }
              />
              <SettingRow
                title="Act"
                description="Open apps and perform reversible computer actions."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.act}
                    onChange={(decision) => void onPermissionChange("act", decision)}
                    label="Act permission"
                  />
                }
              />
              <SettingRow
                title="Modify"
                description="Change application state, type text or activate controls."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.modify}
                    onChange={(decision) => void onPermissionChange("modify", decision)}
                    label="Modify permission"
                  />
                }
              />
              <SettingRow
                title="Sensitive"
                description="Session or privacy-sensitive actions. Permanent Allow is intentionally unavailable."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.sensitive}
                    onChange={(decision) => void onPermissionChange("sensitive", decision)}
                    allowPermanent={false}
                    label="Sensitive permission"
                  />
                }
              />
              <SettingRow
                title="Destructive"
                description="High-impact actions. Permanent Allow is intentionally unavailable."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.destructive}
                    onChange={(decision) => void onPermissionChange("destructive", decision)}
                    allowPermanent={false}
                    label="Destructive permission"
                  />
                }
              />
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Policy</SectionLabel>
              <SettingRow
                title="Safe defaults"
                description="Read and Act are allowed; Modify, Sensitive and Destructive require confirmation."
                trailing={
                  <button
                    type="button"
                    className="settings-action-button"
                    onClick={() => void onResetPermissions()}
                  >
                    Reset
                  </button>
                }
              />
              <SettingRow
                title="Storage"
                description="Permission policy is stored locally on this PC."
                trailing={<Badge tone="ready">Local</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "models" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Models</h2>
              <p>Local and optional cloud intelligence used by AURA.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Runtime</SectionLabel>
              <SettingRow
                title="Local language model"
                description="AURA-2 model runtime has not been connected to the desktop generation yet."
                trailing={<Badge tone="planned">Not configured</Badge>}
              />
              <SettingRow
                title="Model Router"
                description="Routes simple actions directly and uses AI only when reasoning is required."
                trailing={<Badge tone="planned">Planned</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "voice" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Voice</h2>
              <p>Speech input, output and wake-word controls.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Voice</SectionLabel>
              <SettingRow title="Push to talk" description="Talk to AURA without opening the main window." trailing={<Badge tone="planned">M006</Badge>} />
              <SettingRow title="Wake word" description="Optional hands-free activation." trailing={<Badge tone="planned">M006</Badge>} />
              <SettingRow title="Voice output" description="Natural spoken responses for actions and status." trailing={<Badge tone="planned">M006</Badge>} />
            </Surface>
          </>
        )}

        {activeSection === "overlay" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Overlay</h2>
              <p>Quick access to AURA from anywhere on Windows.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Behaviour</SectionLabel>
              <SettingRow title="AURA Overlay" description="Compact always-on-top command surface." trailing={<Badge tone="ready">Enabled</Badge>} />
              <SettingRow title="Hide on focus loss" description="Automatically dismiss the Overlay when you return to another app." trailing={<Badge tone="ready">Enabled</Badge>} />
              <SettingRow title="Overlay shortcut" description="Show or hide AURA instantly." trailing={<span className="settings-keys"><ShortcutKey>Ctrl</ShortcutKey><span>+</span><ShortcutKey>Shift</ShortcutKey><span>+</span><ShortcutKey>Space</ShortcutKey></span>} />
            </Surface>
          </>
        )}

        {activeSection === "shortcuts" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Shortcuts</h2>
              <p>Keyboard access to AURA.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Global shortcuts</SectionLabel>
              <SettingRow
                title="Open AURA Overlay"
                description="Available anywhere while AURA is running."
                trailing={<span className="settings-keys"><ShortcutKey>Ctrl</ShortcutKey><span>+</span><ShortcutKey>Shift</ShortcutKey><span>+</span><ShortcutKey>Space</ShortcutKey></span>}
              />
              <SettingRow
                title="Custom shortcuts"
                description="Shortcut editing and conflict detection arrive later in M002."
                trailing={<Badge tone="planned">Planned</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "integrations" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Integrations</h2>
              <p>Apps and services AURA can control directly.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Available & planned</SectionLabel>
              <SettingRow title="OBS Studio" description="Scene, stream, recording and production control through OBS WebSocket." trailing={<Badge tone="planned">M004</Badge>} />
              <SettingRow title="Windows" description="Native app, window, input and system controls." trailing={<Badge tone="planned">M003</Badge>} />
              <SettingRow title="Future Skills" description="Modular app integrations built on the AURA Skills architecture." trailing={<Badge tone="planned">Later</Badge>} />
            </Surface>
          </>
        )}
      </div>
    </section>
  );
}

export type { SettingsSection };
