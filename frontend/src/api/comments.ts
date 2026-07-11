import type {
    ApiComment,
    CommentLikeResponse,
    CommentsResponse,
} from "@/types/comment";
import client from "./client";

const articleCommentsPath = (articleId: string) =>
    `/api/articles/${encodeURIComponent(articleId)}/comments`;
const commentPath = (commentId: string) =>
    `/api/comments/${encodeURIComponent(commentId)}`;

export const createComment = (
    articleId: string,
    content: string,
    parentId?: string
) =>
    client.post<ApiComment>(articleCommentsPath(articleId), {
        content,
        parent_id: parentId,
    });

export const deleteComment = (commentId: string) =>
    client.delete(commentPath(commentId));

export const getComments = (articleId: string) =>
    client.get<CommentsResponse>(articleCommentsPath(articleId));

export const setCommentLike = (commentId: string, liked: boolean) =>
    client.put<CommentLikeResponse>(`${commentPath(commentId)}/like`, { liked });
