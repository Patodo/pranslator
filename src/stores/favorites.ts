import { create } from 'zustand';
import type { FavoriteItem } from '../types';
import { getFavorites, addFavorite, deleteFavorite } from '../api/translate';

interface FavoritesState {
  favorites: FavoriteItem[];
  isLoading: boolean;
  error: string | null;
  loadFavorites: () => Promise<void>;
  addFavorite: (original: string, translated: string) => Promise<FavoriteItem | null>;
  deleteFavorite: (id: string) => Promise<void>;
}

export const useFavoritesStore = create<FavoritesState>((set) => ({
  favorites: [],
  isLoading: false,
  error: null,

  loadFavorites: async () => {
    set({ isLoading: true, error: null });
    try {
      const favorites = await getFavorites();
      set({ favorites, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  addFavorite: async (original: string, translated: string) => {
    try {
      const item = await addFavorite(original, translated);
      set((state) => ({
        favorites: [item, ...state.favorites],
      }));
      return item;
    } catch (error) {
      set({ error: String(error) });
      return null;
    }
  },

  deleteFavorite: async (id: string) => {
    try {
      await deleteFavorite(id);
      set((state) => ({
        favorites: state.favorites.filter((item) => item.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
