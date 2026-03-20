import { useState } from 'react';
import { translate } from '../api/translate';

export function TranslationPanel() {
  const [inputText, setInputText] = useState('');
  const [outputText, setOutputText] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleTranslate = async () => {
    if (!inputText.trim()) return;

    setIsLoading(true);
    setError(null);

    try {
      const result = await translate({ text: inputText });
      setOutputText(result.translated_text);
    } catch (err) {
      setError(String(err));
      setOutputText('');
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && e.ctrlKey) {
      handleTranslate();
    }
  };

  const handleSwap = () => {
    setInputText(outputText);
    setOutputText('');
  };

  const handleClear = () => {
    setInputText('');
    setOutputText('');
    setError(null);
  };

  return (
    <div className="translation-panel">
      <div className="input-section">
        <textarea
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Enter text to translate (Ctrl+Enter to translate)"
          rows={6}
        />
      </div>

      <div className="button-section">
        <button onClick={handleTranslate} disabled={isLoading || !inputText.trim()}>
          {isLoading ? 'Translating...' : 'Translate'}
        </button>
        <button onClick={handleSwap} disabled={!outputText}>
          Swap
        </button>
        <button onClick={handleClear} disabled={!inputText && !outputText}>
          Clear
        </button>
      </div>

      {error && <div className="error-message">{error}</div>}

      <div className="output-section">
        <textarea
          value={outputText}
          readOnly
          placeholder="Translation result will appear here..."
          rows={6}
        />
      </div>
    </div>
  );
}
