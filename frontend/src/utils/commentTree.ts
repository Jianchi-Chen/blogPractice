import type { ApiComment } from "@/types/comment";

export type CommentNode = Omit<ApiComment, "like_count" | "liked_by_me"> & {
    like_count: number;
    liked_by_me: number;
    children: CommentNode[];
};

export const buildCommentTree = (comments: ApiComment[]): CommentNode[] => {
    const nodes = new Map<string, CommentNode>();

    for (const comment of comments) {
        nodes.set(comment.comment_id, {
            ...comment,
            like_count: comment.like_count ?? 0,
            liked_by_me: comment.liked_by_me ?? 0,
            children: [],
        });
    }

    const roots: CommentNode[] = [];
    for (const comment of comments) {
        const node = nodes.get(comment.comment_id);
        if (!node) continue;

        const parent = comment.parent_id
            ? nodes.get(comment.parent_id)
            : undefined;
        if (parent) parent.children.push(node);
        else roots.push(node);
    }

    return roots;
};

export const findCommentNode = (
    comments: CommentNode[],
    commentId: string
): CommentNode | undefined => {
    for (const comment of comments) {
        if (comment.comment_id === commentId) return comment;
        const child = findCommentNode(comment.children, commentId);
        if (child) return child;
    }
};
