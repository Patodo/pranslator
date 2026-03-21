import { useEffect, useState } from 'react';
import { Cloud, Keyboard } from 'lucide-react';
import { useSettingsStore } from '../stores/settings';
import type { Settings } from '../types';

type TabId = 'api' | 'shortcuts';

const tabs: { id: TabId; label: string; icon: typeof Cloud }[] = [
  { id: 'api', label: 'API', icon: Cloud },
  { id: 'shortcuts', label: 'Shortcuts', icon: Keyboard },
];

export function SettingsPanel() {
  const { settings, loadSettings, updateSettings, isLoading, error } = useSettingsStore();
  const [localSettings, setLocalSettings] = useState<Settings | null>(null);
  const [activeTab, setActiveTab] = useState<TabId>('api');

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    if (settings) {
      setLocalSettings(settings);
    }
  }, [settings]);

  const handleSave = () => {
    if (localSettings) {
      updateSettings(localSettings);
    }
  };

  const handleLlmChange = (field: keyof Settings['llm'], value: string) => {
    if (localSettings) {
      setLocalSettings({
        ...localSettings,
        llm: { ...localSettings.llm, [field]: value },
      });
    }
  };

  const handleShortcutChange = (field: keyof Settings['shortcuts'], value: string) => {
    if (localSettings) {
      setLocalSettings({
        ...localSettings,
        shortcuts: { ...localSettings.shortcuts, [field]: value },
      });
    }
  };

  if (isLoading) return <div className="loading">Loading settings...</div>;
  if (!localSettings) return <div className="loading">No settings available</div>;

  return (
    <div className="settings-panel">
      <div className="settings-tabs">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              className={`settings-tab ${activeTab === tab.id ? 'active' : ''}`}
              onClick={() => setActiveTab(tab.id)}
            >
              <Icon size={16} />
              <span>{tab.label}</span>
            </button>
          );
        })}
      </div>

      <div className="settings-content">
        {error && <div className="error-message">{error}</div>}

        {activeTab === 'api' && (
          <>
            <div className="form-group">
              <label htmlFor="api_key">API Key</label>
              <input
                id="api_key"
                type="password"
                value={localSettings.llm.api_key}
                onChange={(e) => handleLlmChange('api_key', e.target.value)}
                placeholder="sk-..."
              />
            </div>

            <div className="form-group">
              <label htmlFor="api_base">API Base URL</label>
              <input
                id="api_base"
                type="text"
                value={localSettings.llm.api_base}
                onChange={(e) => handleLlmChange('api_base', e.target.value)}
                placeholder="https://api.openai.com/v1"
              />
              <span className="form-hint">
                {localSettings.llm.api_base.replace(/\/+$/, '')}/chat/completions
              </span>
            </div>

            <div className="form-group">
              <label htmlFor="model">Model</label>
              <input
                id="model"
                type="text"
                value={localSettings.llm.model}
                onChange={(e) => handleLlmChange('model', e.target.value)}
                placeholder="gpt-4o-mini"
              />
              <span className="form-hint">The model to use for translation</span>
            </div>
          </>
        )}

        {activeTab === 'shortcuts' && (
          <>
            <div className="form-group">
              <label htmlFor="toggle_window">Toggle Window</label>
              <input
                id="toggle_window"
                type="text"
                value={localSettings.shortcuts.toggle_window}
                onChange={(e) => handleShortcutChange('toggle_window', e.target.value)}
                placeholder="Alt+Shift+T"
              />
              <span className="form-hint">
                Global shortcut to show/hide the application window
              </span>
            </div>
          </>
        )}

        <button className="save-btn" onClick={handleSave}>
          Save Settings
        </button>
      </div>
    </div>
  );
}
