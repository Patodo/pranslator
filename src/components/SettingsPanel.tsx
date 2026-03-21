import { useEffect, useState } from 'react';
import { useSettingsStore } from '../stores/settings';
import type { Settings } from '../types';

export function SettingsPanel() {
  const { settings, loadSettings, updateSettings, isLoading, error } = useSettingsStore();
  const [localSettings, setLocalSettings] = useState<Settings | null>(null);

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

  const handleChange = (field: keyof Settings['llm'], value: string) => {
    if (localSettings) {
      setLocalSettings({
        ...localSettings,
        llm: { ...localSettings.llm, [field]: value },
      });
    }
  };

  if (isLoading) return <div className="loading">Loading settings...</div>;
  if (!localSettings) return <div className="loading">No settings available</div>;

  return (
    <div className="settings-panel">
      <h3>API Settings</h3>

      {error && <div className="error-message">{error}</div>}

      <div className="form-group">
        <label htmlFor="api_key">API Key</label>
        <input
          id="api_key"
          type="password"
          value={localSettings.llm.api_key}
          onChange={(e) => handleChange('api_key', e.target.value)}
          placeholder="sk-..."
        />
      </div>

      <div className="form-group">
        <label htmlFor="api_base">API Base URL</label>
        <input
          id="api_base"
          type="text"
          value={localSettings.llm.api_base}
          onChange={(e) => handleChange('api_base', e.target.value)}
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
          onChange={(e) => handleChange('model', e.target.value)}
          placeholder="gpt-4o-mini"
        />
      </div>

      <button onClick={handleSave}>Save Settings</button>
    </div>
  );
}
