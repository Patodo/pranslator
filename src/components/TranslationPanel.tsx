import { useState } from 'react';
import { translate } from '../api/translate';

export type TranslationStatus = 'idle' | 'loading' | 'success' | 'error';

export function TranslationPanel() {
  const [inputText, setInputText] = useState('');
  const [outputText, setOutputText] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState<TranslationStatus>('idle');

  const handleTranslate = async () => {
    if (!inputText.trim()) return;

    setIsLoading(true);
    setStatus('loading');

    try {
      const result = await translate({ text: inputText });
      setOutputText(result.translated_text);
      setStatus('success');
    } catch (err) {
      setOutputText(String(err));
      setStatus('error');
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
    setStatus('idle');
  };

  const handleClear = () => {
    setInputText('');
    setOutputText('');
    setStatus('idle');
  };

  return {
    inputText,
    outputText,
    isLoading,
    status,
    setInputText,
    handleTranslate,
    handleSwap,
    handleClear,
    handleKeyDown,
  };
}

export function TranslationView({
  inputText,
  outputText,
  status,
  setInputText,
  handleKeyDown,
}: {
  inputText: string;
  outputText: string;
  status: TranslationStatus;
  setInputText: (text: string) => void;
  handleKeyDown: (e: React.KeyboardEvent) => void;
}) {
  const getOutputClass = () => {
    if (status === 'loading') return 'output-loading';
    if (status === 'error') return 'output-error';
    return '';
  };

  const getOutputValue = () => {
    if (status === 'loading') return 'Translating...';
    return outputText;
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

      <div className="output-section">
        <textarea
          className={getOutputClass()}
          value={getOutputValue()}
          readOnly
          placeholder="Translation result will appear here..."
          rows={6}
        />
      </div>
    </div>
  );
}
