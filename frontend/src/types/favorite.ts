import type { Article } from "@/types/article";

export interface FavoriteListResponse {
    articles: Article[];
}

export interface FavoriteState {
    article_id: string;
    favorited: boolean;
}
