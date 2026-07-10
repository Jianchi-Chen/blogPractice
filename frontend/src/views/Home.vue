<template>
    <n-layout class="min-h-full">
        <n-layout-content class="px-6 py-8 max-w-5xl mx-auto">
            <n-space vertical :size="32">
                <!-- 页面标题与筛选区域 -->
                <n-space vertical :size="16">
                    <n-h1
                        class="!mb-0"
                        :style="{ fontSize: '2rem', fontWeight: 600 }"
                    >
                        <n-gradient-text type="success">
                            文章列表
                        </n-gradient-text>
                    </n-h1>

                    <!-- 筛选工具栏 -->
                    <ArticleFilter
                        v-model="select_value"
                        :tag-options="select_options"
                    />
                </n-space>

                <!-- 文章列表 -->
                <n-spin :show="loading" size="large">
                    <n-empty
                        v-if="!loading && articles.length === 0"
                        description="暂无文章"
                        size="large"
                        class="py-12"
                    />

                    <n-space vertical :size="16" v-else>
                        <ArticleCard
                            v-for="article in articles"
                            :key="article.id"
                            :article="article"
                            @click="goToDetail"
                        />
                    </n-space>
                </n-spin>
            </n-space>
        </n-layout-content>
    </n-layout>
</template>

<script setup lang="ts">
import {
    NSpace,
    useMessage,
    NH1,
    NEmpty,
    NSpin,
    NGradientText,
    NLayout,
    NLayoutContent,
} from "naive-ui";
import { fetchArticles } from "@/api/article";
import { onMounted, ref, watch, type Ref } from "vue";
import { useRouter } from "vue-router";
import type { Article } from "@/types/article";
import { useSearchStore } from "@/stores/search";
import ArticleFilter from "@/components/article/ArticleFilter.vue";
import ArticleCard from "@/components/article/ArticleCard.vue";
import type { SelectOption } from "naive-ui";

// 文章列表
const router = useRouter();
const articles = ref<any[]>([]);
const loading = ref(true);
const message = useMessage();
const select_value = ref<string[]>([]);
const search = useSearchStore();

// 筛选项
const select_options: Ref<SelectOption[]> = ref([]);

// 获取文章列表
const loadArticles = async () => {
    loading.value = true;
    try {
        const res = await fetchArticles(search.condition);
        articles.value = res.data.articles; // default is a object

        console.log(articles.value);

        getTags();
    } catch (err) {
        message.error("无法加载文章, 请刷新", {
            duration: 0, // 设置为 0 表示永不自动关闭
            closable: true, // 加一个关闭按钮以防无法关闭
        });
    } finally {
        loading.value = false;
    }
};

// 页面加载时请求
onMounted(() => {
    loadArticles();
});

// 跳转到详情
const goToDetail = (id: number | string) => {
    router.push(`/article/${id}`);
};

// 获取并且更新标签
const getTags = () => {
    // copilot优化，获取tag
    const tagSet = new Set<string>();

    for (const i of articles.value) {
        tagSet.add(i.tags);
    }
    // sort()升序，map()将每一个tag字符串转换为一个对象
    select_options.value = Array.from(tagSet)
        .sort()
        .map((tag) => ({
            label: tag,
            value: tag,
        }));
};

// 根据选中标签过滤文章（不在函数内修改 `select_value`，避免触发循环）
const applyTagFilter = (selected: string[] | undefined) => {
    const sel = Array.isArray(selected) ? selected.map(String) : [];

    if (!sel.length) {
        // 未选中任何标签，恢复完整列表
        loadArticles();
        return;
    }

    const newArticles: Article[] = articles.value.filter((a: any) =>
        sel.includes(String(a.tags))
    );

    articles.value = newArticles;
    console.log("筛选后的文章列表: ", articles.value);
    getTags();
};

// 监听 select_value 的变化来触发筛选
watch(select_value, (newVal) => {
    applyTagFilter(newVal as string[]);
});

// 仅在搜索条件变更时重新加载文章
watch(
    () => search.condition,
    () => {
        loadArticles();
    }
);
</script>
