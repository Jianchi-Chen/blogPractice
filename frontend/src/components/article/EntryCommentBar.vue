<template>
    <n-alert title="提示" v-if="!userhasLogin">登录后才能发表评论</n-alert>
    <n-flex v-else>
        <n-form
            :model="formData"
            :rules="formRules"
            ref="formRef"
            class="w-[90%]"
        >
            <n-form-item path="newComment">
                <n-input
                    ref="inputRef"
                    type="textarea"
                    :autosize="{
                        minRows: 2,
                        maxRows: 6,
                    }"
                    v-model:value="formData.newComment"
                    round
                    placeholder="feel free to feedback..."
                    bordered
                />
                <n-button
                    :loading="loading"
                    @click="submitComment"
                    round
                    type="primary"
                >
                    {{
                        commentCoolDown > 0
                            ? `${commentCoolDown}秒冷却`
                            : "发表评论"
                    }}
                </n-button>
            </n-form-item>
        </n-form>
    </n-flex>
</template>

<script setup lang="ts">
import { useUserStore } from "@/stores/user";
import { ref, watchEffect, nextTick } from "vue";
import { useMessage, type FormInst } from "naive-ui";
import { createComment } from "@/api/comments";

const props = defineProps<{
    articleId: string;
    dataChannel?: string;
}>();
const emit = defineEmits<{
    success: [];
}>();

const message = useMessage();
const userStore = useUserStore();
const userhasLogin = userStore.token ? true : false;
const newComment = ref();
const loading = ref(false);
const commentCoolDown = ref(0);
const timer = ref(0);
const formRef = ref<FormInst | null>();
const inputRef = ref<HTMLElement | null>(null);
const commentParentId = ref<string | null>(null);

const formData = ref({
    newComment: "",
});
const formRules = {
    newComment: {
        required: true,
        min: 2,
        message: "最短评论长度为 2",
    },
};

// 发布评论
const submitComment = async () => {
    loading.value = true;
    try {
        await formRef.value?.validate();
        await createComment(
            props.articleId,
            formData.value.newComment,
            commentParentId.value ? commentParentId.value : undefined
        );
        message.success("评论成功");
        emit("success");
    } catch (e) {
        if (e instanceof Error) {
            message.error(`${e}`);
        }
    } finally {
        // 开始倒计时
        commentCoolDown.value = 3;
        timer.value = setInterval(() => {
            commentCoolDown.value--;
            if (commentCoolDown.value <= 0) {
                clearInterval(timer.value as number);
            }
        }, 1000);

        formData.value.newComment = "";
        setTimeout(() => {
            loading.value = false;
        }, 3000);
    }
};

// 监听 dataChannel 变化，更新评论内容
watchEffect(() => {
    if (props.dataChannel) {
        formData.value.newComment = props.dataChannel;
    }
});

// 用于父组件通过 ref 直接设置评论内容并聚焦输入框
const setComment = async (text: string, parent_id: string) => {
    formData.value.newComment = text;
    commentParentId.value = parent_id;
    // 聚焦到输入框（若组件有 focus 方法）
    await nextTick();
    try {
        // Naive UI Input 实例上有 focus 方法
        (inputRef.value as any)?.focus?.();
    } catch (e) {
        console.error("无法聚焦到输入框:", e);
    }
};

// 暴露 setComment 方法给父组件
defineExpose({ setComment });
</script>

<style scoped></style>
