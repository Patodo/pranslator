import { useState, useEffect, useCallback } from 'react';
import { X, Settings, ArrowRightLeft, Trash2, Languages, Book } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';
import { TranslationPanel, TranslationView } from './components/TranslationPanel';
import { SettingsPanel } from './components/SettingsPanel';
import { FavoritesPanel } from './components/FavoritesPanel';
import { LeaderKeyOverlay } from './components/LeaderKeyOverlay';
import { useLeaderKey } from './hooks/useLeaderKey';
import { useFavoritesStore } from './stores/favorites';
import { hideWindow } from './api/translate';
import { EVENTS } from './constants';
import { DURATIONS } from './constants/animations';
import './App.css';

type PageState = 'home' | 'settings' | 'settings-to-home' | 'favorites' | 'favorites-to-home';

function App() {
  const [pageState, setPageState] = useState<PageState>('home');
  const {
    inputText,
    outputText,
    isLoading,
    status,
    successState,
    copyState,
    favoriteState,
    setInputText,
    handleTranslate,
    handleSwap,
    handleClear,
    handleKeyDown,
    handleCopy,
    handleFavorite,
  } = TranslationPanel();

  const addFavorite = useFavoritesStore((state) => state.addFavorite);

  const goToSettings = () => {
    setPageState('settings');
  };

  const goToHome = () => {
    if (pageState === 'settings' || pageState === 'settings-to-home') {
      setPageState('settings-to-home');
    } else {
      setPageState('favorites-to-home');
    }
  };

  const goToFavorites = () => {
    setPageState('favorites');
  };

  // Reset to home page when window is shown via shortcut
  useEffect(() => {
    const unlisten = listen(EVENTS.RESET_TO_HOME, () => {
      setPageState('home');
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (pageState === 'settings-to-home' || pageState === 'favorites-to-home') {
      const timer = setTimeout(() => setPageState('home'), DURATIONS.PAGE_TRANSITION);
      return () => clearTimeout(timer);
    }
  }, [pageState]);

  // Alt+B shortcut for quick favorite
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.altKey && e.key === 'b' && pageState === 'home' && outputText && inputText) {
        if (favoriteState !== 'saved') {
          addFavorite(inputText, outputText);
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [pageState, inputText, outputText, favoriteState, addFavorite]);

  const showSettings = pageState === 'settings' || pageState === 'settings-to-home';
  const showFavorites = pageState === 'favorites' || pageState === 'favorites-to-home';

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
      ) : showFavorites ? (
        <div className={`page-wrapper ${pageState === 'favorites' ? 'slide-in' : 'slide-out'}`}>
          <div className="toolbar">
            <div className="header-spacer" />
            <h2>Favorites</h2>
            <button className="icon-btn" onClick={goToHome}>
              <X size={20} />
            </button>
          </div>
          <div className="page-content">
            <FavoritesPanel />
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
            <div className="toolbar-right">
              <button className="icon-btn" onClick={goToFavorites} title="Favorites">
                <Book size={20} />
              </button>
              <button className="icon-btn" onClick={goToSettings} title="Settings">
                <Settings size={20} />
              </button>
            </div>
          </div>
          <div className="page-content">
            <TranslationView
              inputText={inputText}
              outputText={outputText}
              status={status}
              successState={successState}
              copyState={copyState}
              favoriteState={favoriteState}
              setInputText={setInputText}
              handleKeyDown={handleKeyDown}
              handleCopy={handleCopy}
              handleFavorite={handleFavorite}
            />
          </div>
        </div>
      )}
    </main>
  );
}

export default App;
