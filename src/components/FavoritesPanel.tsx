import { useEffect, useState } from 'react';
import { Copy, Trash2 } from 'lucide-react';
import { useFavoritesStore } from '../stores/favorites';
import type { FavoriteItem } from '../types';

function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

function FavoriteCard({
  item,
  onDelete,
  onCopy,
}: {
  item: FavoriteItem;
  onDelete: (id: string) => void;
  onCopy: (text: string) => void;
}) {
  const [copyState, setCopyState] = useState<'original' | 'translated' | null>(null);

  const handleCopy = async (text: string, type: 'original' | 'translated') => {
    await onCopy(text);
    setCopyState(type);
    setTimeout(() => setCopyState(null), 1500);
  };

  return (
    <div className="favorite-card">
      <div className="favorite-content">
        <div className="favorite-row">
          <span className="favorite-original">{item.original_text}</span>
          <button
            className={`favorite-copy-btn ${copyState === 'original' ? 'copied' : ''}`}
            onClick={() => handleCopy(item.original_text, 'original')}
            title="Copy original"
          >
            {copyState === 'original' ? (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
              <Copy size={14} />
            )}
          </button>
        </div>
        <div className="favorite-row">
          <span className="favorite-translated">{item.translated_text}</span>
          <button
            className={`favorite-copy-btn ${copyState === 'translated' ? 'copied' : ''}`}
            onClick={() => handleCopy(item.translated_text, 'translated')}
            title="Copy translation"
          >
            {copyState === 'translated' ? (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
              <Copy size={14} />
            )}
          </button>
        </div>
      </div>
      <div className="favorite-footer">
        <span className="favorite-time">{formatTimestamp(item.created_at)}</span>
        <button className="favorite-delete-btn" onClick={() => onDelete(item.id)} title="Delete">
          <Trash2 size={14} />
        </button>
      </div>
    </div>
  );
}

export function FavoritesPanel() {
  const { favorites, isLoading, loadFavorites, deleteFavorite } = useFavoritesStore();

  useEffect(() => {
    loadFavorites();
  }, [loadFavorites]);

  const handleCopy = async (text: string) => {
    await navigator.clipboard.writeText(text);
  };

  const handleDelete = async (id: string) => {
    await deleteFavorite(id);
  };

  if (isLoading) {
    return (
      <div className="favorites-panel">
        <div className="favorites-loading">Loading...</div>
      </div>
    );
  }

  if (favorites.length === 0) {
    return (
      <div className="favorites-panel">
        <div className="favorites-empty">No favorites yet</div>
      </div>
    );
  }

  return (
    <div className="favorites-panel">
      <div className="favorites-list">
        {favorites.map((item) => (
          <FavoriteCard key={item.id} item={item} onDelete={handleDelete} onCopy={handleCopy} />
        ))}
      </div>
    </div>
  );
}
