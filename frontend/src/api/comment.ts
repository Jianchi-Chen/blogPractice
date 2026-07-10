import client from "./client";
import { useAppStore } from "@/stores/app";
import { useUserStore } from "@/stores/user";
import { invoke } from "@tauri-apps/api/core";
import type {
    ApiComment,
    CommentLikeResponse,
    CommentsResponse,
} from "@/types/comment";

// 发表评论
export const postComment = async (
    article_id: string,
    content: string,
    user_id: string,
    parent_id?: string
) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke<ApiComment>("post_comment", {
            token: user.token,
            commentData: {
                article_id,
                content,
                parent_id
            }
        });
        return { data };
    }
    
    return client.post("/api/comment", { article_id, user_id, content, parent_id });
};

// 删除评论
export const DeleteComment = async (CommentId: string) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke("delete_comment", {
            token: user.token,
            commentId: CommentId
        });
        return { data };
    }
    
    return client.delete(`/comment/${CommentId}`);
};

// 获取评论
export const fetchComments = async (articleId: string) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke<CommentsResponse>("get_comments", {
            articleId,
            token: user.token || null
        });
        return { data };
    }
    
    return client.get<CommentsResponse>(`/comments/${articleId}`);
};

// 获取评论点赞情况
export const fetchCommentsLike = async (articleId: string) => {
    const app = useAppStore();
    
    if (app.isTauri) {
        // TODO: 如果需要实现 Tauri 端的点赞获取，需要添加对应的命令
        // 暂时保持 Web 端行为
    }
    
    return client.get("/api/comments/like", {
        params: { article_id: articleId },
    });
};

// 更新评论点赞数
export const updateCommentLike = async (commentId: string, userToken: string) => {
    const app = useAppStore();
    const user = useUserStore();
    
    if (app.isTauri) {
        const data = await invoke<CommentLikeResponse>("like_comment", {
            token: user.token,
            payload: {
                comment_id: commentId
            }
        });
        return { data };
    }
    
    return client.put<CommentLikeResponse>("/api/comment/like", {
        comment_id: commentId,
    });
};
