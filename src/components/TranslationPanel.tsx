import { useState, useRef, useEffect, useCallback } from 'react';
import { translate } from '../api/translate';
import { DURATIONS } from '../constants/animations';

export type TranslationStatus = 'idle' | 'loading' | 'success' | 'error';
export type CopyState = 'idle' | 'copied';

export function TranslationPanel() {
  const [inputText, setInputText] = useState('');
  const [outputText, setOutputText] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState<TranslationStatus>('idle');
  const [successState, setSuccessState] = useState<'none' | 'show' | 'fade'>('none');
  const [copyState, setCopyState] = useState<CopyState>('idle');

  const handleTranslate = async () => {
    if (!inputText.trim()) return;

    setIsLoading(true);
    setStatus('loading');
    setOutputText('');

    try {
      const result = await translate({ text: inputText });
      setOutputText(result.translated_text);
      setStatus('success');
      setSuccessState('show');
      setTimeout(() => setSuccessState('fade'), DURATIONS.SUCCESS_FADE_START);
      setTimeout(() => setSuccessState('none'), DURATIONS.SUCCESS_FADE_END);
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

  const handleCopy = useCallback(async () => {
    if (!outputText) return;
    await navigator.clipboard.writeText(outputText);
    setCopyState('copied');
    setTimeout(() => setCopyState('idle'), DURATIONS.COPY_FEEDBACK);
  }, [outputText]);

  return {
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
  };
}

export function TranslationView({
  inputText,
  outputText,
  status,
  successState,
  copyState,
  setInputText,
  handleKeyDown,
  handleCopy,
}: {
  inputText: string;
  outputText: string;
  status: TranslationStatus;
  successState: 'none' | 'show' | 'fade';
  copyState: CopyState;
  setInputText: (text: string) => void;
  handleKeyDown: (e: React.KeyboardEvent) => void;
  handleCopy: () => void;
}) {
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const outputRef = useRef<HTMLTextAreaElement>(null);
  const [allowOutputFocus, setAllowOutputFocus] = useState(false);

  useEffect(() => {
    const el = inputRef.current;
    if (el) {
      el.focus();
      el.selectionStart = el.selectionEnd = el.value.length;
    }
  }, []);

  const handleOutputDoubleClick = useCallback(() => {
    setAllowOutputFocus(true);
    setTimeout(() => outputRef.current?.focus(), 0);
  }, []);

  const handleOutputBlur = useCallback(() => {
    setAllowOutputFocus(false);
    const el = inputRef.current;
    if (el) {
      el.focus();
      el.selectionStart = el.selectionEnd = el.value.length;
    }
  }, []);

  const getOutputClass = () => {
    if (status === 'loading') return 'output-loading';
    if (status === 'error') return 'output-error';
    if (successState === 'show') return 'output-success';
    if (successState === 'fade') return 'output-success fade-out';
    return '';
  };

  const getOutputValue = () => {
    return outputText;
  };

  const getPlaceholder = () => {
    if (status === 'loading') return 'Translating...';
    return 'Translation result will appear here...';
  };

  return (
    <div className="translation-panel">
      <div className="input-section">
        <textarea
          ref={inputRef}
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={() => {
            if (allowOutputFocus) return;
            const el = inputRef.current;
            if (el) {
              el.focus();
              el.selectionStart = el.selectionEnd = el.value.length;
            }
          }}
          placeholder="Enter text to translate (Ctrl+Enter to translate)"
          rows={6}
        />
      </div>

      <div className="output-section">
        <div className="output-wrapper">
          <textarea
            ref={outputRef}
            className={getOutputClass()}
            value={getOutputValue()}
            readOnly
            placeholder={getPlaceholder()}
            rows={6}
            onDoubleClick={handleOutputDoubleClick}
            onBlur={handleOutputBlur}
          />
          {outputText && (
            <button
              className={`copy-btn ${copyState === 'copied' ? 'copied' : ''}`}
              onClick={handleCopy}
              title={copyState === 'copied' ? 'Copied!' : 'Copy'}
            >
              {copyState === 'copied' ? (
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              ) : (
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
