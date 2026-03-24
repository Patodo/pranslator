import { useCallback, useEffect, useState } from 'react';
import { SHORTCUTS } from '../config/shortcuts';
import { DURATIONS } from '../constants/animations';

interface LeaderKeyOverlayProps {
  isVisible: boolean;
}

export function LeaderKeyOverlay({ isVisible }: LeaderKeyOverlayProps) {
  const [shouldRender, setShouldRender] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);

  useEffect(() => {
    if (isVisible) {
      setShouldRender(true);
      requestAnimationFrame(() => {
        setIsAnimating(true);
      });
    } else {
      setIsAnimating(false);
      const timer = setTimeout(() => {
        setShouldRender(false);
      }, DURATIONS.NORMAL);
      return () => clearTimeout(timer);
    }
  }, [isVisible]);

  const handleBackdropClick = useCallback(() => {
    // Clicking backdrop does nothing - user must press a key
  }, []);

  if (!shouldRender) return null;

  return (
    <div
      className={`leader-key-overlay ${isAnimating ? 'visible' : ''}`}
      onClick={handleBackdropClick}
    >
      <div className="leader-key-card">
        <div className="leader-key-header">
          <span className="leader-key-badge">Alt</span>
          <span className="leader-key-plus">+</span>
          <span className="leader-key-hint">key</span>
        </div>
        <div className="leader-key-shortcuts">
          {SHORTCUTS.map((shortcut) => (
            <div key={shortcut.key} className="leader-key-item">
              <span className="leader-key-badge">{shortcut.key.toUpperCase()}</span>
              <span className="leader-key-desc">{shortcut.description}</span>
            </div>
          ))}
        </div>
        <div className="leader-key-footer">
          Press <kbd>Esc</kbd> to cancel
        </div>
      </div>
    </div>
  );
}
