import { useState } from 'react';
import { ArrowLeft, Settings, ArrowRightLeft, Trash2, Languages } from 'lucide-react';
import { TranslationPanel, TranslationView } from './components/TranslationPanel';
import { SettingsPanel } from './components/SettingsPanel';
import './App.css';

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const {
    inputText,
    outputText,
    isLoading,
    status,
    setInputText,
    handleTranslate,
    handleSwap,
    handleClear,
    handleKeyDown,
  } = TranslationPanel();

  return (
    <main className="app-container">
      {showSettings ? (
        <>
          <div className="toolbar">
            <button className="icon-btn" onClick={() => setShowSettings(false)}>
              <ArrowLeft size={20} />
            </button>
            <h2>Settings</h2>
            <div className="header-spacer" />
          </div>
          <div className="page-content">
            <SettingsPanel />
          </div>
        </>
      ) : (
        <>
          <div className="toolbar">
            <div className="toolbar-left">
              <button
                className="primary-btn"
                onClick={handleTranslate}
                disabled={isLoading || !inputText.trim()}
              >
                <Languages size={18} />
                <span>Translate</span>
              </button>
              <button
                className="icon-btn"
                onClick={handleSwap}
                disabled={!outputText}
                title="Swap"
              >
                <ArrowRightLeft size={18} />
              </button>
              <button
                className="icon-btn"
                onClick={handleClear}
                disabled={!inputText && !outputText}
                title="Clear"
              >
                <Trash2 size={18} />
              </button>
            </div>
            <button className="icon-btn" onClick={() => setShowSettings(true)} title="Settings">
              <Settings size={20} />
            </button>
          </div>
          <div className="page-content">
            <TranslationView
              inputText={inputText}
              outputText={outputText}
              status={status}
              setInputText={setInputText}
              handleKeyDown={handleKeyDown}
            />
          </div>
        </>
      )}
    </main>
  );
}

export default App;
