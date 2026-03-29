import { useEffect, useRef, useState } from 'react';
import { Book, Check, Cloud, Download, Keyboard, Trash2, X } from 'lucide-react';
import type { DictionaryStatus, DownloadProgress, Settings } from '../types';
import { DICTIONARY_DOWNLOAD_URL } from '../types';
import {
  cancelDictionaryDownload,
  deleteDictionary,
  downloadDictionary,
  getDictionaryStatus,
} from '../api/dictionary';
import { useSettingsStore } from '../stores/settings';
import { GLOBAL_SHORTCUT_CONFIGS } from '../config/globalShortcuts';
import { ShortcutInput } from './ShortcutInput';

type TabId = 'api' | 'shortcuts' | 'dictionary';

const tabs: { id: TabId; label: string; icon: typeof Cloud }[] = [
  { id: 'api', label: 'API', icon: Cloud },
  { id: 'shortcuts', label: 'Shortcuts', icon: Keyboard },
  { id: 'dictionary', label: 'Dictionary', icon: Book },
];

export function SettingsPanel({ initialTab }: { initialTab?: TabId } = {}) {
  const { settings, loadSettings, updateSettings, isLoading, error } = useSettingsStore();
  const [localSettings, setLocalSettings] = useState<Settings | null>(null);
  const [activeTab, setActiveTab] = useState<TabId>(initialTab ?? 'api');

  const initialTabApplied = useRef(false);

  useEffect(() => {
    if (initialTab && !initialTabApplied.current) {
      setActiveTab(initialTab);
      initialTabApplied.current = true;
    }
  }, [initialTab]);
  const [dictStatus, setDictStatus] = useState<DictionaryStatus>({
    downloaded: false,
    downloading: false,
    progress: 0,
  });
  const [downloadSpeed, setDownloadSpeed] = useState('');

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    if (settings) {
      setLocalSettings(settings);
    }
  }, [settings]);

  // Load dictionary status
  useEffect(() => {
    getDictionaryStatus().then(setDictStatus).catch(console.error);
  }, []);

  const handleDownloadDictionary = async () => {
    setDictStatus((prev) => ({ ...prev, downloading: true, progress: 0 }));
    setDownloadSpeed('');
    try {
      await downloadDictionary((progress: DownloadProgress) => {
        setDictStatus((prev) => ({ ...prev, progress: progress.progress }));
        setDownloadSpeed(progress.speed);
      });
      const status = await getDictionaryStatus();
      setDictStatus(status);
      setDownloadSpeed('');
    } catch (err) {
      console.error('Download failed:', err);
      // Refresh actual status from backend — the file may have been
      // extracted even though Dictionary::open failed afterwards.
      try {
        const status = await getDictionaryStatus();
        setDictStatus(status);
      } catch {
        setDictStatus((prev) => ({ ...prev, downloading: false }));
      }
      setDownloadSpeed('');
    }
  };

  const handleCancelDownload = async () => {
    await cancelDictionaryDownload();
    setDictStatus((prev) => ({ ...prev, downloading: false, progress: 0 }));
    setDownloadSpeed('');
  };

  const handleDeleteDictionary = async () => {
    try {
      await deleteDictionary();
      setDictStatus({ downloaded: false, downloading: false, progress: 0 });
    } catch (err) {
      console.error('Delete failed:', err);
    }
  };

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

  if (isLoading || !localSettings) return null;

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

            <div className="form-group">
              <label htmlFor="system_prompt">Prompt Template</label>
              <textarea
                id="system_prompt"
                value={localSettings.llm.system_prompt}
                onChange={(e) => handleLlmChange('system_prompt', e.target.value)}
                rows={6}
              />
              <span className="form-hint">
                Required: <code>{'{{text}}'}</code> will be replaced with the text to translate
              </span>
            </div>
          </>
        )}

        {activeTab === 'shortcuts' && (
          <div className="shortcuts-list">
            {GLOBAL_SHORTCUT_CONFIGS.map((config) => (
              <div key={config.field} className="shortcut-item">
                <span className="shortcut-name">
                  {config.name}
                  <span className="shortcut-tooltip">{config.description}</span>
                </span>
                <ShortcutInput
                  value={localSettings.shortcuts[config.field]}
                  onChange={(v) => handleShortcutChange(config.field, v)}
                />
              </div>
            ))}
          </div>
        )}

        {activeTab === 'dictionary' && (
          <div className="dictionary-section">
            <div className="dictionary-header">
              <div className="dictionary-icon">
                <Download size={24} />
              </div>
              <div className="dictionary-info">
                <h3>ECDICT Dictionary</h3>
                <p>3.4M entries with phonetics</p>
              </div>
            </div>

            <div className="dictionary-card">
              {dictStatus.downloaded ? (
                <>
                  <div className="dictionary-status-row">
                    <div className="status-badge success">
                      <Check size={14} />
                      <span>Downloaded</span>
                    </div>
                    <span className="file-size">{dictStatus.fileSize}</span>
                  </div>
                  <p className="dictionary-hint">
                    Word translation will use offline dictionary for faster results.
                  </p>
                  <button
                    className="dictionary-action-btn danger"
                    onClick={handleDeleteDictionary}
                    disabled={dictStatus.downloading}
                  >
                    <Trash2 size={16} />
                    <span>Delete Dictionary</span>
                  </button>
                </>
              ) : dictStatus.downloading ? (
                <>
                  <div className="download-progress">
                    <div className="progress-header">
                      <span>Downloading...</span>
                      <span className="progress-percent">{dictStatus.progress}%</span>
                    </div>
                    <div className="progress-track">
                      <div className="progress-fill" style={{ width: `${dictStatus.progress}%` }} />
                    </div>
                    {downloadSpeed && <div className="progress-speed">{downloadSpeed}</div>}
                  </div>
                  <p className="dictionary-hint">Downloading from GitHub. Please wait...</p>
                  <button className="dictionary-action-btn danger" onClick={handleCancelDownload}>
                    <X size={16} />
                    <span>Cancel Download</span>
                  </button>
                </>
              ) : (
                <>
                  <div className="dictionary-status-row">
                    <div className="status-badge">
                      <span>Not Downloaded</span>
                    </div>
                    <span className="file-size">~93 MB</span>
                  </div>
                  <p className="dictionary-hint">
                    Download for faster offline word translation with phonetics.
                  </p>
                  <button className="dictionary-action-btn" onClick={handleDownloadDictionary}>
                    <Download size={16} />
                    <span>Download Dictionary</span>
                  </button>
                  <div className="manual-download-hint">
                    <p>
                      Or download manually:{' '}
                      <a href={DICTIONARY_DOWNLOAD_URL} target="_blank" rel="noopener noreferrer">
                        {DICTIONARY_DOWNLOAD_URL}
                      </a>
                    </p>
                    <p className="hint-note">
                      Extract the .mdx file to the config directory after downloading.
                    </p>
                  </div>
                </>
              )}
            </div>
          </div>
        )}

        <button className="save-btn" onClick={handleSave}>
          Save Settings
        </button>
      </div>
    </div>
  );
}
