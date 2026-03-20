import { useState } from 'react';
import { TranslationPanel } from './components/TranslationPanel';
import { SettingsPanel } from './components/SettingsPanel';
import './App.css';

type Tab = 'translate' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('translate');

  return (
    <main className="app-container">
      <div className="tab-header">
        <button
          className={activeTab === 'translate' ? 'active' : ''}
          onClick={() => setActiveTab('translate')}
        >
          Translate
        </button>
        <button
          className={activeTab === 'settings' ? 'active' : ''}
          onClick={() => setActiveTab('settings')}
        >
          Settings
        </button>
      </div>

      <div className="tab-content">
        {activeTab === 'translate' && <TranslationPanel />}
        {activeTab === 'settings' && <SettingsPanel />}
      </div>
    </main>
  );
}

export default App;
