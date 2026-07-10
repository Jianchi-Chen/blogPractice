import { ref } from "vue";
import { useMessage } from "naive-ui";
import { useUserStore } from "@/stores/user";
import { fetchComments, updateCommentLike, DeleteComment } from "@/api/comment";

export function useComments() {
    const comments = ref<any[]>([]);
    const ifComment = ref(false);
    const message = useMessage();
    const userStore = useUserStore();

    const buildCommentsTree = (commentsData: any) => {
        const map = new Map();
        const roots: any[] = [];
        const items = Array.isArray(commentsData) ? commentsData : [];

        items.forEach((c) => {
            map.set(c.comment_id, { ...c, children: [] });
        });

        items.forEach((c) => {
            if (c.parent_id == null) {
                roots.push(map.get(c.comment_id));
            } else {
                const parent = map.get(c.parent_id);
                if (parent) parent.children.push(map.get(c.comment_id));
            }
        });

        return roots;
    };

    const loadComments = async (articleId: string) => {
        try {
            const res = await fetchComments(articleId);
            comments.value = buildCommentsTree(res.data.comments);
            ifComment.value = comments.value.length > 0;
        } catch (e) {
            console.error("Error fetching comments:", e);
            message.error("加载评论失败，请稍后重试");
        }
    };

    const postedComment = async (articleId: string) => {
        await loadComments(articleId);
    };

    const handlerDeleteComment = async (commentid: string, articleId?: string) => {
        try {
            await DeleteComment(commentid);
            if (articleId) await loadComments(articleId);
        } catch (e) {
            console.error(e);
            message.error("删除评论失败");
        }
    };

    const likeComment = async (
        commentId: string,
        mode: string,
        articleId?: string
    ) => {
        const utoken = userStore.token;
        if (!utoken) {
            message.warning("请先登录后再点赞");
            return;
        }

        // find target
        let target: any = null;
        if (mode === "main") {
            target = comments.value.find((c) => c.comment_id === commentId);
        } else {
            target = comments.value
                .flatMap((c) => c.children)
                .find((child: any) => child.comment_id === commentId);
        }

        if (!target) {
            message.error("找不到对应的评论");
            return;
        }

        const previousLikedState = target.liked_by_me;
        const previousLikeCount = target.like_count;

        if (previousLikedState === 0) {
            target.like_count += 1;
            target.liked_by_me = 1;
        } else {
            target.like_count -= 1;
            target.liked_by_me = 0;
        }

        try {
            const res = await updateCommentLike(commentId, utoken);
            const responseData = res.data.like_or_unlike;

            const isLiked = responseData === "liked";
            const expectedState = previousLikedState === 0 ? 1 : 0;

            if ((isLiked ? 1 : 0) !== expectedState) {
                target.like_count = previousLikeCount;
                target.liked_by_me = previousLikedState;
                message.error("点赞状态异常，请刷新页面重试");
                console.error("状态不一致:", {
                    isLiked,
                    expectedState,
                    previousLikedState,
                    responseData,
                });
                return;
            }

            if (isLiked) message.success("点赞成功");
            else message.info("取消点赞");
        } catch (e) {
            target.like_count = previousLikeCount;
            target.liked_by_me = previousLikedState;
            message.error("操作失败，请稍后重试");
            console.error("点赞失败:", e);
        }
    };

    return {
        comments,
        ifComment,
        loadComments,
        postedComment,
        handlerDeleteComment,
        likeComment,
    };
}
