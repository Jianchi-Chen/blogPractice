import type {
    FavoriteListResponse,
    FavoriteState,
} from "@/types/favorite";
import client from "./client";

const favoritePath = (articleId: string) =>
    `/api/favorites/${encodeURIComponent(articleId)}`;

export const getFavorites = () =>
    client.get<FavoriteListResponse>("/api/favorites");

export const addFavorite = (articleId: string) =>
    client.put<FavoriteState>(favoritePath(articleId));

export const removeFavorite = (articleId: string) =>
    client.delete<FavoriteState>(favoritePath(articleId));
