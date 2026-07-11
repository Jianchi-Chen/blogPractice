<template>
    <div>
        <n-form
            :model="form"
            :rules="rules"
            ref="formRef"
            label-placement="top"
        >
            <n-form-item label="标题" path="title">
                <n-input
                    v-model:value="form.title"
                    round
                    placeholder="请输入文章标题"
                />
            </n-form-item>

            <!-- 文章标签 -->
            <n-form-item path="tags" :show-label="false">
                <n-flex size="large">
                    <n-text class="m-8">文章标签*1: </n-text>
                    <n-dynamic-tags
                        v-model:value="form.tags"
                        :max="1"
                        placeholder="输入标签后回车"
                    />
                </n-flex>
            </n-form-item>

            <!-- 文章摘要 -->
            <n-form-item label="摘要" path="summary">
                <n-input
                    v-model:value="form.summary"
                    type="textarea"
                    :rows="3"
                    round
                    placeholder="请输入文章摘要"
                />
            </n-form-item>

            <n-form-item label="内容" path="content">
                <!-- vue鼓励组件嵌套 -->
                <!-- 父组件只有一个 v-model 且不带参数时，等价于：v-model:modelValue -->
                <MarkDownEditor
                    v-model:modelValue="form.content"
                    style="width: 80%; height: 400px"
                />
            </n-form-item>

            <n-form-item>
                <n-button
                    type="primary"
                    @click="handleSubmit"
                    :loading="loading"
                    >{{ isEdit ? "暂存修改" : "保存文章" }}</n-button
                >
            </n-form-item>
        </n-form>
    </div>
</template>

<script setup lang="ts">
import {
    computed,
    onBeforeUnmount,
    onMounted,
    ref,
    watch,
    type Ref,
} from "vue";
import {
    NForm,
    NFormItem,
    NInput,
    NButton,
    useMessage,
    NCard,
    NDynamicTags,
    NText,
    NFlex,
} from "naive-ui";
import axios from "@/api/client";
import { useRouter } from "vue-router";
import { createArticle, updateArticle } from "@/api/articles";
import type { Article } from "@/types/article";
import Vditor from "vditor";
import { tr } from "zod/locales";
import MarkDownEditor from "@/components/common/MarkdownEditor.vue";

// props类型约束
const props = defineProps<{
    article: Article; // 这里貌似不能传入ref<Article>，换成普通的Article后报错消失，不过表单内可以自己包一层
    isEdit?: boolean;
    articleId?: Article["id"];
}>();

// emit
const emit = defineEmits<{
    (e: "done"): void;
}>();

// 表单数据。直接赋值会把 form 指向同一个 ref，建议深拷贝
// 多定义一个tags会覆盖原有tags，这里强定义tags不为undefined
const form = ref({
    ...props.article,
    tags: props.article.tags ? [props.article.tags] : ["Universal"],
});

// 校验规则
const rules = {
    title: {
        required: true, // 表示该字段是必填项
        message: "请输入标题", // 当用户未填写时显示的提示信息
        trigger: ["input", "blur"], // 表示在“输入”和“失焦”时触发验证
    },
    content: {
        required: true,
        message: "请输入内容",
        trigger: ["input", "blur"],
    },
    tags: {
        trigger: ["change"],
        validator(rule: unknown, value: string[]) {
            if (value.length >= 5) {
                throw new Error("不得超过四个标签");
            }

            return true;
        },
    },
};

const message = useMessage();
const router = useRouter();
const formRef = ref();
const loading = ref(false);

// 初始化表单内容并监听字段
watch(
    () => props.article,
    (val) => {
        // 手动处理 tags的值
        if (val)
            form.value = {
                ...val,
                tags: val.tags ? [val.tags] : ["Universal"],
            };
    },
    { immediate: true }
);

// 提交表单
const handleSubmit = async () => {
    try {
        // naive ui 自带的表单校验
        await formRef.value?.validate();
        // console.log("await formRef.value?.validate()  //!  -1")
        loading.value = true;
        const outcomeForm = { ...form.value, tags: form.value.tags[0] };

        if (props.isEdit && props.articleId) {
            // 编辑模式

            outcomeForm.status = "draft";
            await updateArticle(props.articleId, outcomeForm);
            message.success("Changed Successfully");
        } else {
            // 发布模式
            outcomeForm.status = "draft";
            if (!outcomeForm.tags) {
                outcomeForm.tags = "Universal";
            }
            message.success("save Successfully");
            await createArticle({ ...outcomeForm });
        }
        emit("done");
        router.push("/admin");
    } catch (e) {
        if (e instanceof Error) {
            message.error(`${e}`);
        }
        console.error(e);
        message.error("保存失败");
    } finally {
        loading.value = false;
    }
};
</script>
