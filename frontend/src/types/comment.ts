import z, { boolean } from "zod";

export const CommentSchema = z.object({
    id: z.string().optional(),
    user: z.string().optional(),
    content: z.string().min(1, "不能为空"),
    created_at: z.string().optional(),
    praise_count: z.number().optional(), // 点赞数
    replied: z.boolean().optional(), // 是否为回复的评论
    parent_id: z.string().optional(), // 父评论ID
});

export type Comment = z.infer<typeof CommentSchema>;

export interface ApiComment {
    comment_id: string;
    article_id?: string;
    user?: string;
    content?: string;
    created_at?: string;
    parent_id?: string;
    like_count?: number;
    liked_by_me?: number;
}

export interface CommentsResponse {
    comments: ApiComment[];
}

export interface CommentLikeResponse {
    comment_id: string;
    like_or_unlike: "liked" | "unliked";
}

export const createEmptyComment = (): Comment => {
    return CommentSchema.parse({
        content: "unknown comment",
    });
};
