import type {
    Article,
    ArticleListResponse,
    SuggestionResponse,
} from "@/types/article";
import client from "./client";

const articlePath = (id: Article["id"]) =>
    `/api/articles/${encodeURIComponent(String(id))}`;

export const getArticles = (query?: string) =>
    client.get<ArticleListResponse>("/api/articles", {
        params: query ? { query } : undefined,
    });

export const getArticle = (id: Article["id"]) =>
    client.get<Article>(articlePath(id));

export const createArticle = (input: Article) =>
    client.post<Article>("/api/articles", input);

export const updateArticle = (id: Article["id"], input: Article) =>
    client.put<Article>(articlePath(id), input);

export const deleteArticle = (id: Article["id"]) =>
    client.delete(articlePath(id));

export const updateArticleStatus = (
    id: Article["id"],
    status: "draft" | "published" | "archived"
) => client.patch<Article>(`${articlePath(id)}/status`, { status });

export const getArticleSuggestions = (query: string, signal?: AbortSignal) =>
    client.get<SuggestionResponse>("/api/search/suggestions", {
        params: { query },
        signal,
    });
