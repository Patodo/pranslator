import { useState, useEffect, useCallback } from 'react';
import { X, Settings, ArrowRightLeft, Trash2, Languages } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';
import { TranslationPanel, TranslationView } from './components/TranslationPanel';
import { SettingsPanel } from './components/SettingsPanel';
import { LeaderKeyOverlay } from './components/LeaderKeyOverlay';
import { useLeaderKey } from './hooks/useLeaderKey';
import { hideWindow } from './api/translate';
import './App.css';

type PageState = 'home' | 'settings' | 'settings-to-home';

function App() {
  const [pageState, setPageState] = useState<PageState>('home');
  const {
    inputText,
    outputText,
    isLoading,
    status,
    successState,
    copyState,
    setInputText,
    handleTranslate,
    handleSwap,
    handleClear,
    handleKeyDown,
    handleCopy,
  } = TranslationPanel();

  const goToSettings = () => {
    setPageState('settings');
  };

  const goToHome = () => {
    setPageState('settings-to-home');
  };

  // Reset to home page when window is shown via shortcut
  useEffect(() => {
    const unlisten = listen('reset-to-home', () => {
      setPageState('home');
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (pageState === 'settings-to-home') {
      const timer = setTimeout(() => setPageState('home'), 200);
      return () => clearTimeout(timer);
    }
  }, [pageState]);

  const showSettings = pageState !== 'home';

  const handleHideWindow = useCallback(async () => {
    await hideWindow();
  }, []);

  const leaderKeyHandlers = useCallback(
    () => ({
      translate: handleTranslate,
      copy: handleCopy,
      copyAndHide: async () => {
        await handleCopy();
        await handleHideWindow();
      },
      hide: handleHideWindow,
    }),
    [handleTranslate, handleCopy, handleHideWindow]
  );

  const { isLeaderMode } = useLeaderKey({
    outputText,
    handlers: leaderKeyHandlers(),
    enabled: !showSettings,
  });

  return (
    <main className={`app-container ${pageState}`}>
      <LeaderKeyOverlay isVisible={isLeaderMode} />
      {showSettings ? (
        <div className={`page-wrapper ${pageState === 'settings' ? 'slide-in' : 'slide-out'}`}>
          <div className="toolbar">
            <div className="header-spacer" />
            <h2>Settings</h2>
            <button className="icon-btn" onClick={goToHome}>
              <X size={20} />
            </button>
          </div>
          <div className="page-content">
            <SettingsPanel />
          </div>
        </div>
      ) : (
        <div className="page-wrapper">
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
              <button className="icon-btn" onClick={handleSwap} disabled={!outputText} title="Swap">
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
            <button className="icon-btn" onClick={goToSettings} title="Settings">
              <Settings size={20} />
            </button>
          </div>
          <div className="page-content">
            <TranslationView
              inputText={inputText}
              outputText={outputText}
              status={status}
              successState={successState}
              copyState={copyState}
              setInputText={setInputText}
              handleKeyDown={handleKeyDown}
              handleCopy={handleCopy}
            />
          </div>
        </div>
      )}
    </main>
  );
}

export default App;
