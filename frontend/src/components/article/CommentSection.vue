<template>
    <n-layout class="w-[95%] comment-container">
        <div ref="commentTitle">
            <n-h2>评论区</n-h2>
        </div>
        <!-- 评论框 -->
        <div
            :class="{ fixedBottom: isFixed }"
            :style="{
                left: isFixed
                    ? articleStore.expandFolder
                        ? '64px'
                        : '240px'
                    : 'auto',

                background: articleStore.osTheme === false ? '#fff' : '#1f1f1f',
            }"
        >
            <EntryCommentBar
                ref="entryRef"
                :articleId="articleId"
                @success="onPosted"
            />
        </div>

        <n-alert v-if="!ifComment" type="info" show-icon class="mb-4">
            暂无评论，快来抢沙发吧~
        </n-alert>

        <n-card v-for="comment in comments" :key="comment.comment_id">
            <CommentItem
                :comment="comment"
                @like="(id, mode) => handleLike(id, mode)"
                @reply="
                    (username, parentId) => respondComment(username, parentId)
                "
                @delete="(id) => confirmDelete(id)"
            />
        </n-card>
    </n-layout>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { useDialog, NLayout, NCard, NAlert } from "naive-ui";
import { useArticleStore } from "@/stores/article";
import EntryCommentBar from "./EntryCommentBar.vue";
import CommentItem from "./CommentItem.vue";
import { useComments } from "@/composables/useComments";

const props = defineProps<{ articleId: string }>();

const commentTitle = ref<HTMLElement | null>(null);
const isFixed = ref(false);
const articleStore = useArticleStore();
const dialog = useDialog();
const entryRef = ref<any>(null);
let observer: IntersectionObserver | undefined;

const {
    comments,
    ifComment,
    loadComments,
    postedComment,
    handlerDeleteComment,
    likeComment,
} = useComments();

// 初始化和 DOM observer
onMounted(() => {
    observer = new IntersectionObserver(
        ([entry]) => {
            isFixed.value = !entry.isIntersecting;
        },
        { threshold: 0 }
    );

    if (commentTitle.value) observer.observe(commentTitle.value);
});

onUnmounted(() => observer?.disconnect());

const respondComment = (username: string, parent_id: string) => {
    entryRef.value?.setComment?.(`@${username} `, parent_id);
};

const onPosted = async () => {
    if (props.articleId) await postedComment(props.articleId);
};

const handleLike = (commentId: string, mode: string) => {
    likeComment(commentId, mode, props.articleId);
};

const confirmDelete = (commentId: string) => {
    dialog.warning({
        title: "确认删除?",
        content: "此操作将永久删除该评论，是否继续?",
        positiveText: "确定",
        negativeText: "取消",
        onPositiveClick: async () => {
            await handlerDeleteComment(commentId, props.articleId);
        },
    });
};

watch(
    () => props.articleId,
    (articleId) => {
        if (articleId) loadComments(articleId);
    },
    { immediate: true }
);
</script>

<style scoped>
.fixedBottom {
    position: fixed;
    bottom: 0;
    right: 20px;
    z-index: 1000;
    transition: left 0.3s ease;
}

/* 防止评论框遮住最后一条评论，添加底部间距 */
.comment-container {
    padding-bottom: 280px;
}

/* 子评论样式 - 贴合 Naive UI 风格 */
.child-comment {
    margin-top: 12px;
    margin-bottom: 12px;
    margin-left: 32px;
    padding: 12px 16px;
    border-left: 3px solid var(--n-border-color, #e0e0e0);
    border-radius: 2px;
    background: var(--n-color, #fafafa);
    transition: background-color 0.3s ease, border-color 0.3s ease;
}
</style>
