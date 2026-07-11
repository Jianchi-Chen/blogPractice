import { ref } from "vue";
import { useMessage } from "naive-ui";
import { useUserStore } from "@/stores/user";
import { deleteComment, getComments, setCommentLike } from "@/api/comments";
import {
    buildCommentTree,
    findCommentNode,
    type CommentNode,
} from "@/utils/commentTree";

export function useComments() {
    const comments = ref<CommentNode[]>([]);
    const ifComment = ref(false);
    const message = useMessage();
    const userStore = useUserStore();
    let latestLoadRequest = 0;
    const pendingLikes = new Set<string>();

    const loadComments = async (articleId: string) => {
        const requestId = ++latestLoadRequest;
        comments.value = [];
        ifComment.value = false;
        try {
            const res = await getComments(articleId);
            if (requestId !== latestLoadRequest) return;
            comments.value = buildCommentTree(res.data.comments);
            ifComment.value = comments.value.length > 0;
        } catch (e) {
            if (requestId !== latestLoadRequest) return;
            console.error("Error fetching comments:", e);
            message.error("加载评论失败，请稍后重试");
        }
    };

    const reloadComments = async (articleId: string) => {
        await loadComments(articleId);
    };

    const removeComment = async (commentId: string, articleId?: string) => {
        try {
            await deleteComment(commentId);
            if (articleId) await loadComments(articleId);
        } catch (e) {
            console.error(e);
            message.error("删除评论失败");
        }
    };

    const toggleCommentLike = async (commentId: string) => {
        const utoken = userStore.token;
        if (!utoken) {
            message.warning("请先登录后再点赞");
            return;
        }

        const target = findCommentNode(comments.value, commentId);

        if (!target) {
            message.error("找不到对应的评论");
            return;
        }

        if (pendingLikes.has(commentId)) return;
        pendingLikes.add(commentId);
        const previousLikedState = target.liked_by_me;
        const previousLikeCount = target.like_count;
        const shouldLike = previousLikedState !== 1;
        target.liked_by_me = shouldLike ? 1 : 0;
        target.like_count = Math.max(0, previousLikeCount + (shouldLike ? 1 : -1));

        try {
            const res = await setCommentLike(commentId, shouldLike);
            target.liked_by_me = res.data.liked_by_me ? 1 : 0;
            target.like_count = res.data.like_count;
            if (res.data.liked_by_me) message.success("点赞成功");
            else message.info("取消点赞");
        } catch (e) {
            target.like_count = previousLikeCount;
            target.liked_by_me = previousLikedState;
            message.error("操作失败，请稍后重试");
            console.error("点赞失败:", e);
        } finally {
            pendingLikes.delete(commentId);
        }
    };

    return {
        comments,
        ifComment,
        loadComments,
        reloadComments,
        removeComment,
        toggleCommentLike,
    };
}
