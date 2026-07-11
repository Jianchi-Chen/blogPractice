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
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import type { Article } from "@/types/article";
import { useSearchStore } from "@/stores/search";
import ArticleFilter from "@/components/article/ArticleFilter.vue";
import ArticleCard from "@/components/article/ArticleCard.vue";
import type { SelectOption } from "naive-ui";

// 文章列表
const router = useRouter();
const allArticles = ref<Article[]>([]);
const loading = ref(true);
const message = useMessage();
const select_value = ref<string[]>([]);
const search = useSearchStore();

const articles = computed(() => {
    if (!select_value.value.length) return allArticles.value;
    return allArticles.value.filter((article) =>
        select_value.value.includes(article.tags || "Universal")
    );
});

// 筛选项始终来自完整结果集，筛选不会破坏原始文章数据。
const select_options = computed<SelectOption[]>(() =>
    Array.from(
        new Set(
            allArticles.value.map(
                (article) => article.tags || "Universal"
            )
        )
    )
        .sort()
        .map((tag) => ({ label: tag, value: tag }))
);

let latestLoadRequest = 0;

// 获取文章列表
const loadArticles = async () => {
    const requestId = ++latestLoadRequest;
    loading.value = true;
    try {
        const res = await fetchArticles(search.condition);
        if (requestId !== latestLoadRequest) return;
        allArticles.value = res.data.articles;
    } catch (err) {
        if (requestId !== latestLoadRequest) return;
        message.error("无法加载文章, 请刷新", {
            duration: 0, // 设置为 0 表示永不自动关闭
            closable: true, // 加一个关闭按钮以防无法关闭
        });
    } finally {
        if (requestId === latestLoadRequest) {
            loading.value = false;
        }
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

// 仅在搜索条件变更时重新加载文章
watch(
    () => search.condition,
    () => {
        loadArticles();
    }
);
</script>
